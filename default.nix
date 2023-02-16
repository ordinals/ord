{ lib, fetchFromGitHub , rustPlatform , pkg-config, openssl }:

rustPlatform.buildRustPackage rec {
  pname = "ord";
  version = "0.5.0";

  src = fetchFromGitHub {
    owner = "casey";
    repo = pname;
    rev = version;
    sha256 = "sha256-0atd/zW4NsTAmnJYjjlnB9rWnvK7d/ap9RgFRPqNrpc=";
  };

  cargoSha256 = "sha256-vuMPk7TDXeyL4w/igO0IG6KSG2uzJftartEOWoMUjO8";

  nativeBuildInputs = [pkg-config];
  buildInputs = [openssl];

  meta = with lib; {
    description = "ord is an index, block explorer, and command-line wallet. Ordinal theory imbues satoshis with numismatic value, allowing them to collected and traded as curios.";
    homepage = "https://github.com/casey/ord";
    license = with licenses; [cc0];
    mainProgram = "ord";
  };
}

