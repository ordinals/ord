# Copyright 2019 Google LLC. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# ==============================================================================
"""Converts an image between PNG and TFCI formats.

Use this script to compress images with pre-trained models as published. See the
'models' subcommand for a list of available models.

This script requires TFC v2 (`pip install tensorflow-compression==2.*`).
"""

import argparse
import io
import os
import sys
import urllib
from absl import app
from absl.flags import argparse_flags
import numpy as np
import tensorflow as tf
import tensorflow_compression as tfc  # pylint:disable=unused-import


# Default URL to fetch metagraphs from.
URL_PREFIX = "https://storage.googleapis.com/tensorflow_compression/metagraphs"
# Default location to store cached metagraphs.
METAGRAPH_CACHE = "/tmp/tfc_metagraphs"


def read_png(filename):
  """Loads a PNG image file."""
  string = tf.io.read_file(filename)
  image = tf.image.decode_image(string, channels=3)
  return tf.expand_dims(image, 0)


def write_png(filename, image):
  """Writes a PNG image file."""
  image = tf.squeeze(image, 0)
  if image.dtype.is_floating:
    image = tf.round(image)
  if image.dtype != tf.uint8:
    image = tf.saturate_cast(image, tf.uint8)
  string = tf.image.encode_png(image)
  tf.io.write_file(filename, string)


def load_cached(filename):
  """Downloads and caches files from web storage."""
  pathname = os.path.join(METAGRAPH_CACHE, filename)
  try:
    with tf.io.gfile.GFile(pathname, "rb") as f:
      string = f.read()
  except tf.errors.NotFoundError:
    url = f"{URL_PREFIX}/{filename}"
    request = urllib.request.urlopen(url)
    try:
      string = request.read()
    finally:
      request.close()
    tf.io.gfile.makedirs(os.path.dirname(pathname))
    with tf.io.gfile.GFile(pathname, "wb") as f:
      f.write(string)
  return string


def instantiate_model_signature(model, signature, inputs=None, outputs=None):
  """Imports a trained model and returns one of its signatures as a function."""
  string = load_cached(model + ".metagraph")
  metagraph = tf.compat.v1.MetaGraphDef()
  metagraph.ParseFromString(string)
  wrapped_import = tf.compat.v1.wrap_function(
      lambda: tf.compat.v1.train.import_meta_graph(metagraph), [])
  graph = wrapped_import.graph
  if inputs is None:
    inputs = metagraph.signature_def[signature].inputs
    inputs = [graph.as_graph_element(inputs[k].name) for k in sorted(inputs)]
  else:
    inputs = [graph.as_graph_element(t) for t in inputs]
  if outputs is None:
    outputs = metagraph.signature_def[signature].outputs
    outputs = [graph.as_graph_element(outputs[k].name) for k in sorted(outputs)]
  else:
    outputs = [graph.as_graph_element(t) for t in outputs]
  return wrapped_import.prune(inputs, outputs)


def compress_image(model, input_image, rd_parameter=None):
  """Compresses an image tensor into a bitstring."""
  sender = instantiate_model_signature(model, "sender")
  if len(sender.inputs) == 1:
    if rd_parameter is not None:
      raise ValueError("This model doesn't expect an RD parameter.")
    tensors = sender(input_image)
  elif len(sender.inputs) == 2:
    if rd_parameter is None:
      raise ValueError("This model expects an RD parameter.")
    rd_parameter = tf.constant(rd_parameter, dtype=sender.inputs[1].dtype)
    tensors = sender(input_image, rd_parameter)
    # Find RD parameter and expand it to a 1D tensor so it fits into the
    # PackedTensors format.
    for i, t in enumerate(tensors):
      if t.dtype.is_floating and t.shape.rank == 0:
        tensors[i] = tf.expand_dims(t, 0)
  else:
    raise RuntimeError("Unexpected model signature.")
  packed = tfc.PackedTensors()
  packed.model = model
  packed.pack(tensors)
  return packed.string


def compress(model, input_file, output_file,
             rd_parameter=None, rd_parameter_tolerance=None,
             target_bpp=None, bpp_strict=False):
  """Compresses a PNG file to a TFCI file."""
  if not output_file:
    output_file = input_file + ".tfci"

  # Load image.
  input_image = read_png(input_file)
  num_pixels = input_image.shape[-2] * input_image.shape[-3]

  if not target_bpp:
    # Just compress with a specific model.
    bitstring = compress_image(model, input_image, rd_parameter=rd_parameter)
  else:
    # Get model list.
    models = load_cached(model + ".models")
    models = models.decode("ascii").split()

    try:
      lower, upper = [float(m) for m in models]
      use_rd_parameter = True
    except ValueError:
      lower = -1
      upper = len(models)
      use_rd_parameter = False

    # Do a binary search over RD points.
    bpp = None
    best_bitstring = None
    best_bpp = None
    while bpp != target_bpp:
      if use_rd_parameter:
        if upper - lower <= rd_parameter_tolerance:
          break
        i = (upper + lower) / 2
        bitstring = compress_image(model, input_image, rd_parameter=i)
      else:
        if upper - lower < 2:
          break
        i = (upper + lower) // 2
        bitstring = compress_image(models[i], input_image)
      bpp = 8 * len(bitstring) / num_pixels
      is_admissible = bpp <= target_bpp or not bpp_strict
      is_better = (best_bpp is None or
                   abs(bpp - target_bpp) < abs(best_bpp - target_bpp))
      if is_admissible and is_better:
        best_bitstring = bitstring
        best_bpp = bpp
      if bpp < target_bpp:
        lower = i
      if bpp > target_bpp:
        upper = i
    if best_bpp is None:
      assert bpp_strict
      raise RuntimeError(
          "Could not compress image to less than {} bpp.".format(target_bpp))
    bitstring = best_bitstring

  # Write bitstring to disk.
  with tf.io.gfile.GFile(output_file, "wb") as f:
    f.write(bitstring)


def decompress(input_file, output_file):
  """Decompresses a TFCI file and writes a PNG file."""
  if not output_file:
    output_file = input_file + ".png"
  with tf.io.gfile.GFile(input_file, "rb") as f:
    packed = tfc.PackedTensors(f.read())
  receiver = instantiate_model_signature(packed.model, "receiver")
  tensors = packed.unpack([t.dtype for t in receiver.inputs])
  # Find potential RD parameter and turn it back into a scalar.
  for i, t in enumerate(tensors):
    if t.dtype.is_floating and t.shape == (1,):
      tensors[i] = tf.squeeze(t, 0)
  output_image, = receiver(*tensors)
  write_png(output_file, output_image)


def list_models():
  """Lists available models in web storage with a description."""
  url = URL_PREFIX + "/models.txt"
  request = urllib.request.urlopen(url)
  try:
    print(request.read().decode("utf-8"))
  finally:
    request.close()


def list_tensors(model):
  """Lists all internal tensors of a given model."""
  def get_names_dtypes_shapes(function):
    for op in function.graph.get_operations():
      for tensor in op.outputs:
        yield tensor.name, tensor.dtype.name, tensor.shape

  sender = instantiate_model_signature(model, "sender")
  tensors = sorted(get_names_dtypes_shapes(sender))
  print("Sender-side tensors:")
  for name, dtype, shape in tensors:
    print(f"{name} (dtype={dtype}, shape={shape})")
  print()

  receiver = instantiate_model_signature(model, "receiver")
  tensors = sorted(get_names_dtypes_shapes(receiver))
  print("Receiver-side tensors:")
  for name, dtype, shape in tensors:
    print(f"{name} (dtype={dtype}, shape={shape})")


def dump_tensor(model, tensors, input_file, output_file):
  """Dumps the given tensors of a model in .npz format."""
  if not output_file:
    output_file = input_file + ".npz"
  # Note: if receiver-side tensors are requested, this is no problem, as the
  # metagraph contains the union of the sender and receiver graphs.
  sender = instantiate_model_signature(model, "sender", outputs=tensors)
  input_image = read_png(input_file)
  # Replace special characters in tensor names with underscores.
  table = str.maketrans(r"^./-:", r"_____")
  tensors = [t.translate(table) for t in tensors]
  values = [t.numpy() for t in sender(input_image)]
  assert len(tensors) == len(values)
  # Write to buffer first, since GFile might not be random accessible.
  with io.BytesIO() as buf:
    np.savez(buf, **dict(zip(tensors, values)))
    with tf.io.gfile.GFile(output_file, mode="wb") as f:
      f.write(buf.getvalue())


def parse_args(argv):
  """Parses command line arguments."""
  parser = argparse_flags.ArgumentParser(
      formatter_class=argparse.ArgumentDefaultsHelpFormatter)

  # High-level options.
  parser.add_argument(
      "--url_prefix",
      default=URL_PREFIX,
      help="URL prefix for downloading model metagraphs.")
  parser.add_argument(
      "--metagraph_cache",
      default=METAGRAPH_CACHE,
      help="Directory where to cache model metagraphs.")
  subparsers = parser.add_subparsers(
      title="commands", dest="command",
      help="Invoke '<command> -h' for more information.")

  # 'compress' subcommand.
  compress_cmd = subparsers.add_parser(
      "compress",
      formatter_class=argparse.ArgumentDefaultsHelpFormatter,
      description="Reads a PNG file, compresses it using the given model, and "
                  "writes a TFCI file.")
  compress_cmd.add_argument(
      "model",
      help="Unique model identifier. See 'models' command for options. If "
           "'target_bpp' is provided, don't specify the index at the end of "
           "the model identifier.")
  compress_cmd.add_argument(
      "--rd_parameter", "-r", type=float,
      help="Rate-distortion parameter (for some models). Ignored if "
           "'target_bpp' is set.")
  compress_cmd.add_argument(
      "--rd_parameter_tolerance", type=float,
      default=2 ** -4,
      help="Tolerance for rate-distortion parameter. Only used if 'target_bpp' "
           "is set for some models, to determine when to stop the binary "
           "search.")
  compress_cmd.add_argument(
      "--target_bpp", "-b", type=float,
      help="Target bits per pixel. If provided, a binary search is used to try "
           "to match the given bpp as close as possible. In this case, don't "
           "specify the index at the end of the model identifier. It will be "
           "automatically determined.")
  compress_cmd.add_argument(
      "--bpp_strict", action="store_true",
      help="Try never to exceed 'target_bpp'. Ignored if 'target_bpp' is not "
           "set.")

  # 'decompress' subcommand.
  decompress_cmd = subparsers.add_parser(
      "decompress",
      formatter_class=argparse.ArgumentDefaultsHelpFormatter,
      description="Reads a TFCI file, reconstructs the image using the model "
                  "it was compressed with, and writes back a PNG file.")

  # 'models' subcommand.
  subparsers.add_parser(
      "models",
      formatter_class=argparse.ArgumentDefaultsHelpFormatter,
      description="Lists available trained models. Requires an internet "
                  "connection.")

  tensors_cmd = subparsers.add_parser(
      "tensors",
      formatter_class=argparse.ArgumentDefaultsHelpFormatter,
      description="Lists names of internal tensors of a given model.")
  tensors_cmd.add_argument(
      "model",
      help="Unique model identifier. See 'models' command for options.")

  dump_cmd = subparsers.add_parser(
      "dump",
      formatter_class=argparse.ArgumentDefaultsHelpFormatter,
      description="Dumps values of given internal tensors of a model in "
                  "NumPy's .npz format.")
  dump_cmd.add_argument(
      "model",
      help="Unique model identifier. See 'models' command for options.")
  dump_cmd.add_argument(
      "--tensor", "-t", nargs="+",
      help="Name(s) of tensor(s) to dump. Must provide at least one. See "
           "'tensors' command for options.")

  # Arguments for 'compress', 'decompress', and 'dump'.
  for cmd, ext in (
      (compress_cmd, ".tfci"),
      (decompress_cmd, ".png"),
      (dump_cmd, ".npz"),
  ):
    cmd.add_argument(
        "input_file",
        help="Input filename.")
    cmd.add_argument(
        "output_file", nargs="?",
        help=f"Output filename (optional). If not provided, appends '{ext}' to "
             f"the input filename.")

  # Parse arguments.
  args = parser.parse_args(argv[1:])
  if args.command is None:
    parser.print_usage()
    sys.exit(2)
  return args


def main(args):
  # Command line can override these defaults.
  global URL_PREFIX, METAGRAPH_CACHE
  URL_PREFIX = args.url_prefix
  METAGRAPH_CACHE = args.metagraph_cache

  # Invoke subcommand.
  if args.command == "compress":
    compress(args.model, args.input_file, args.output_file,
             args.rd_parameter, args.rd_parameter_tolerance,
             args.target_bpp, args.bpp_strict)
  elif args.command == "decompress":
    decompress(args.input_file, args.output_file)
  elif args.command == "models":
    list_models()
  elif args.command == "tensors":
    list_tensors(args.model)
  elif args.command == "dump":
    if not args.tensor:
      raise ValueError("Must provide at least one tensor to dump.")
    dump_tensor(args.model, args.tensor, args.input_file, args.output_file)


if __name__ == "__main__":
  app.run(main, flags_parser=parse_args)
