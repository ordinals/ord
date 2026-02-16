class HybridOpusPlayer {
  constructor() {
    this.wasmDecoder = null;
  }

  supportsOpusNatively() {
    const audio = document.createElement('audio');
    const canPlay = audio.canPlayType('audio/ogg; codecs="opus"');
    return canPlay === 'probably' || canPlay === 'maybe';
  }

  isSafari() {
    return /^((?!chrome|android).)*safari/i.test(navigator.userAgent);
  }

  isAppleDevice() {
    return /iPad|iPhone|iPod|Mac/.test(navigator.userAgent);
  }

  supportsWebAssembly() {
    return typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function';
  }

  async playOpusAudio(audioElement, opusUrl) {
    const forceTranscoding = this.isSafari();
    const hasNativeSupport = this.supportsOpusNatively() && !forceTranscoding;

    if (hasNativeSupport) {
      return this.playNatively(audioElement, opusUrl);
    } else {
      return this.playWithWasm(audioElement, opusUrl);
    }
  }

  async playNatively(audioElement, opusUrl) {
    if (this.isSafari()) {
      try {
        const response = await fetch(opusUrl);
        const arrayBuffer = await response.arrayBuffer();

        const mimeTypes = [
          'audio/ogg; codecs="opus"',
          'audio/ogg',
          'audio/webm; codecs="opus"',
          'audio/webm',
          'audio/mpeg',
          'audio/wav'
        ];

        for (const mimeType of mimeTypes) {
          try {
            const blob = new Blob([arrayBuffer], { type: mimeType });
            const blobUrl = URL.createObjectURL(blob);

            const testAudio = new Audio();
            testAudio.src = blobUrl;

            const canPlay = testAudio.canPlayType(mimeType);

            if (canPlay === 'probably' || canPlay === 'maybe') {
              audioElement.src = blobUrl;
              audioElement.load();
              return audioElement.play();
            }

            URL.revokeObjectURL(blobUrl);
          } catch (error) {
          }
        }

      } catch (error) {
      }
    }

    audioElement.src = opusUrl;
    audioElement.load();
    return audioElement.play();
  }

  async playWithWasm(audioElement, opusUrl) {
    try {
      if (!this.wasmDecoder) {
        this.wasmDecoder = await this.loadWasmDecoder();
      }

      const opusData = await this.fetchOpusData(opusUrl);

      const wavData = await this.transcodeOpusToWav(opusData);

      const wavBlob = new Blob([wavData], { type: 'audio/wav' });
      const wavUrl = URL.createObjectURL(wavBlob);

      audioElement.src = wavUrl;
      audioElement.load();

      return audioElement.play();

    } catch (error) {
      console.error('WASM transcoding failed:', error);
      throw error;
    }
  }

  async loadRawOpusDecoder() {
    try {
      let DecoderClass;

      if (typeof window.OpusDecoder !== 'undefined') {
        DecoderClass = window.OpusDecoder;
      } else if (window['opus-decoder'] && window['opus-decoder'].OpusDecoder) {
        DecoderClass = window['opus-decoder'].OpusDecoder;
      } else {
        throw new Error('Raw OpusDecoder not found');
      }

      const decoder = new DecoderClass();
      await decoder.ready;

      return decoder;
    } catch (error) {
      console.error('Failed to load raw Opus decoder:', error);
      throw error;
    }
  }

  async loadWasmDecoder() {
    try {
      let attempts = 0;
      while (attempts < 10) {
        if (typeof window.OpusStreamDecoder !== 'undefined') {
          break;
        }
        await new Promise(resolve => setTimeout(resolve, 500));
        attempts++;
      }

      if (typeof window.OpusStreamDecoder === 'undefined') {
        throw new Error('OpusStreamDecoder not found after waiting');
      }

      this.decodedAudio = {
        channelData: [],
        sampleRate: null,
        totalSamples: 0
      };

      const decoder = new window.OpusStreamDecoder({
        onDecode: ({left, right, samplesDecoded, sampleRate}) => {
          if (!this.decodedAudio.sampleRate) {
            this.decodedAudio.sampleRate = sampleRate;
            this.decodedAudio.channelData[0] = [];
            this.decodedAudio.channelData[1] = [];
          }

          this.decodedAudio.channelData[0].push(...left);
          this.decodedAudio.channelData[1].push(...right);
          this.decodedAudio.totalSamples += samplesDecoded;
        }
      });

      return decoder;

    } catch (error) {
      console.error('Failed to load OpusStreamDecoder:', error);
      throw new Error('Could not load OpusStreamDecoder: ' + error.message);
    }
  }

  async fetchOpusData(url) {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const arrayBuffer = await response.arrayBuffer();
    const data = new Uint8Array(arrayBuffer);

    let offset = 0;
    let pageCount = 0;
    while (offset < data.length - 27 && pageCount < 3) {
      if (data[offset] === 0x4F && data[offset + 1] === 0x67 &&
          data[offset + 2] === 0x67 && data[offset + 3] === 0x53) {

        const headerType = data[offset + 5];
        const segmentCount = data[offset + 26];

        if (pageCount === 2) {
          const segmentTableOffset = offset + 27;
          let totalPageSize = 0;
          for (let i = 0; i < segmentCount; i++) {
            totalPageSize += data[segmentTableOffset + i];
          }

          const pageDataOffset = segmentTableOffset + segmentCount;
          const audioData = data.slice(pageDataOffset, pageDataOffset + Math.min(totalPageSize, 20));

          const firstByte = audioData[0];
          const isAllSameByte = audioData.every(byte => byte === firstByte);
          if (isAllSameByte) {
          } else {
          }
        }

        const segmentTableOffset = offset + 27;
        let totalPageSize = 0;
        for (let i = 0; i < segmentCount; i++) {
          totalPageSize += data[segmentTableOffset + i];
        }
        offset = segmentTableOffset + segmentCount + totalPageSize;
        pageCount++;
      } else {
        offset++;
      }
    }

    return arrayBuffer;
  }

  parseOggOpus(opusData) {
    const data = new Uint8Array(opusData);

    if (data[0] !== 0x4F || data[1] !== 0x67 || data[2] !== 0x67 || data[3] !== 0x53) {
      throw new Error('Not a valid OGG file');
    }

    const allPackets = [];
    let offset = 0;
    let pageCount = 0;

    while (offset < data.length - 27) {
      if (data[offset] !== 0x4F || data[offset + 1] !== 0x67 ||
          data[offset + 2] !== 0x67 || data[offset + 3] !== 0x53) {
        offset++;
        continue;
      }

      const version = data[offset + 4];
      const headerType = data[offset + 5];
      const granulePosition = new DataView(data.buffer, offset + 6, 8).getBigUint64(0, true);
      const serialNumber = new DataView(data.buffer, offset + 14, 4).getUint32(0, true);
      const pageSequence = new DataView(data.buffer, offset + 18, 4).getUint32(0, true);
      const checksum = new DataView(data.buffer, offset + 22, 4).getUint32(0, true);
      const segmentCount = data[offset + 26];

      let segmentTableOffset = offset + 27;
      let totalPageSize = 0;
      const segmentSizes = [];

      for (let i = 0; i < segmentCount; i++) {
        const segmentSize = data[segmentTableOffset + i];
        segmentSizes.push(segmentSize);
        totalPageSize += segmentSize;
      }

      let pageDataOffset = segmentTableOffset + segmentCount;

      if (pageCount >= 2) {
        const pageData = data.slice(pageDataOffset, pageDataOffset + Math.min(totalPageSize, 20));
        const pageHeader = Array.from(pageData.slice(0, Math.min(16, pageData.length)))
          .map(b => b.toString(16).padStart(2, '0')).join(' ');

        let segmentOffset = 0;
        let currentPacket = new Uint8Array(0);

        for (let i = 0; i < segmentSizes.length; i++) {
          const segmentSize = segmentSizes[i];
          const segment = data.slice(pageDataOffset + segmentOffset, pageDataOffset + segmentOffset + segmentSize);

          const newPacket = new Uint8Array(currentPacket.length + segment.length);
          newPacket.set(currentPacket);
          newPacket.set(segment, currentPacket.length);
          currentPacket = newPacket;

          if (segmentSize < 255) {
            if (currentPacket.length > 0) {
              allPackets.push(currentPacket);

              currentPacket = new Uint8Array(0);
            }
          }

          segmentOffset += segmentSize;
        }

        if (currentPacket.length > 0) {
          allPackets.push(currentPacket);
        }
      }

      offset = pageDataOffset + totalPageSize;
      pageCount++;
    }

    if (allPackets.length === 0) {
      throw new Error('No Opus packets found in OGG file');
    }

    return allPackets;
  }

  extractOpusPacketsFromOgg(opusData) {
    const data = new Uint8Array(opusData);

    if (data[0] !== 0x4F || data[1] !== 0x67 || data[2] !== 0x67 || data[3] !== 0x53) {
      throw new Error('Not a valid OGG file');
    }

    const rawOpusPackets = [];
    let offset = 0;
    let pageCount = 0;

    while (offset < data.length - 27) {
      if (data[offset] !== 0x4F || data[offset + 1] !== 0x67 ||
          data[offset + 2] !== 0x67 || data[offset + 3] !== 0x53) {
        offset++;
        continue;
      }

      const version = data[offset + 4];
      const headerType = data[offset + 5];
      const granulePosition = new DataView(data.buffer, offset + 6, 8).getBigUint64(0, true);
      const serialNumber = new DataView(data.buffer, offset + 14, 4).getUint32(0, true);
      const pageSequence = new DataView(data.buffer, offset + 18, 4).getUint32(0, true);
      const checksum = new DataView(data.buffer, offset + 22, 4).getUint32(0, true);
      const segmentCount = data[offset + 26];

      const segmentTableOffset = offset + 27;
      let totalPageSize = 0;
      const segmentSizes = [];

      for (let i = 0; i < segmentCount; i++) {
        const segmentSize = data[segmentTableOffset + i];
        segmentSizes.push(segmentSize);
        totalPageSize += segmentSize;
      }

      const pageDataOffset = segmentTableOffset + segmentCount;

      if (pageCount >= 2) {
        let segmentOffset = 0;
        let currentOggPacket = new Uint8Array(0);

        for (let i = 0; i < segmentSizes.length; i++) {
          const segmentSize = segmentSizes[i];
          const segment = data.slice(pageDataOffset + segmentOffset, pageDataOffset + segmentOffset + segmentSize);

          const newOggPacket = new Uint8Array(currentOggPacket.length + segment.length);
          newOggPacket.set(currentOggPacket);
          newOggPacket.set(segment, currentOggPacket.length);
          currentOggPacket = newOggPacket;

          if (segmentSize < 255) {
            if (currentOggPacket.length > 0) {
              if (this.isNonAudioPacket(currentOggPacket)) {
              } else {
                const opusPackets = this.extractRawOpusFromOggPacket(currentOggPacket);
                rawOpusPackets.push(...opusPackets);
              }
              currentOggPacket = new Uint8Array(0);
            }
          }

          segmentOffset += segmentSize;
        }

        if (currentOggPacket.length > 0) {
          if (!this.isNonAudioPacket(currentOggPacket)) {
            const opusPackets = this.extractRawOpusFromOggPacket(currentOggPacket);
            rawOpusPackets.push(...opusPackets);
          } else {
          }
        }
      } else {
      }

      offset = pageDataOffset + totalPageSize;
      pageCount++;
    }

    return rawOpusPackets;
  }

  extractRawOpusFromOggPacket(oggPacket) {
    const opusPackets = [];
    let offset = 0;

    const header = Array.from(oggPacket.slice(0, Math.min(64, oggPacket.length)))
      .map(b => b.toString(16).padStart(2, '0')).join(' ');

    const ascii = Array.from(oggPacket.slice(0, Math.min(64, oggPacket.length)))
      .map(b => (b >= 32 && b <= 126) ? String.fromCharCode(b) : '.')
      .join('');

    while (offset < oggPacket.length) {
      const tocByte = oggPacket[offset];

      const config = (tocByte >> 3) & 0x1F;
      const stereo = (tocByte >> 2) & 0x01;
      const frameCountCode = tocByte & 0x03;

      if (config > 31) {
        offset++;
        continue;
      }

      let packetSize = this.calculateOpusPacketSize(oggPacket, offset, frameCountCode);

      if (packetSize <= 0 || offset + packetSize > oggPacket.length) {
        offset++;
        continue;
      }

      const opusPacket = oggPacket.slice(offset, offset + packetSize);

      const packetHeader = Array.from(opusPacket.slice(0, Math.min(16, opusPacket.length)))
        .map(b => b.toString(16).padStart(2, '0')).join(' ');

      if (this.isValidRawOpusPacket(opusPacket)) {
        opusPackets.push(opusPacket);
        offset += packetSize;
      } else {
        offset++;
      }
    }

    return opusPackets;
  }

  calculateOpusPacketSize(packet, offset, frameCountCode) {
    if (offset >= packet.length) return 0;

    const tocByte = packet[offset];
    const config = (tocByte >> 3) & 0x1F;

    if (frameCountCode === 0) {
      return this.findSelfDelimitedFrameSize(packet, offset);
    } else if (frameCountCode === 1) {
      if (offset + 2 >= packet.length) return 0;
      const length1 = packet[offset + 1];
      const length2 = packet[offset + 2];
      return 1 + length1 + length2;
    } else if (frameCountCode === 2) {
      if (offset + 1 >= packet.length) return 0;
      const length = packet[offset + 1];
      return 1 + 1 + length + length;
    } else if (frameCountCode === 3) {
      if (offset + 1 >= packet.length) return 0;
      const numFrames = packet[offset + 1];
      if (offset + 1 + numFrames >= packet.length) return 0;

      let totalLength = 1 + 1;
      for (let i = 0; i < numFrames; i++) {
        totalLength += packet[offset + 2 + i];
      }
      return totalLength;
    }

    return 0;
  }

  findSelfDelimitedFrameSize(packet, offset) {
    for (let i = offset + 1; i < packet.length; i++) {
      const byte = packet[i];
      const config = (byte >> 3) & 0x1F;
      const frameCountCode = byte & 0x03;

      if (config <= 31 && frameCountCode <= 3) {
        return i - offset;
      }
    }

    return packet.length - offset;
  }

  isValidRawOpusPacket(packet) {
    if (!packet || packet.length < 1) {
      return false;
    }

    const tocByte = packet[0];
    const config = (tocByte >> 3) & 0x1F;
    const frameCountCode = tocByte & 0x03;

    if (config > 31) {
      return false;
    }

    if (packet.length > 10) {
      const isAllSame = packet.slice(0, 10).every(b => b === packet[0]);
      if (isAllSame) {
        return false;
      }

      const isAsciiText = packet.slice(0, Math.min(20, packet.length))
        .every(b => (b >= 32 && b <= 126) || b === 10 || b === 13);
      if (isAsciiText) {
        return false;
      }
    }

    if (packet.length < 10 || packet.length > 4000) {
      return false;
    }

    return true;
  }

  isNonAudioPacket(packet) {
    if (!packet || packet.length === 0) return true;

    const firstByte = packet[0];
    if (packet.length > 100 && packet.slice(0, 100).every(b => b === firstByte)) {
      return true;
    }

    if (packet.length > 50) {
      const sample = packet.slice(0, 50);
      const isBase64Like = sample.every(b =>
        (b >= 0x30 && b <= 0x39) ||
        (b >= 0x41 && b <= 0x5A) ||
        (b >= 0x61 && b <= 0x7A) ||
        b === 0x2B || b === 0x2F || b === 0x3D
      );
      if (isBase64Like) return true;
    }

    return false;
  }

  parseOpusFramesFromOggPacket(packet) {
    const frames = [];
    let offset = 0;

    while (offset < packet.length - 1) {
      const tocByte = packet[offset];

      const config = (tocByte >> 3) & 0x1F;
      const frameCountCode = tocByte & 0x03;

      if (config > 31) {
        offset++;
        continue;
      }

      let frameSize = this.determineOpusFrameSize(packet, offset, frameCountCode);

      if (frameSize <= 0 || offset + frameSize > packet.length) {
        offset++;
        continue;
      }

      const frame = packet.slice(offset, offset + frameSize);

      if (this.isValidOpusFrame(frame)) {
        frames.push(frame);
        offset += frameSize;
      } else {
        offset++;
      }
    }

    return frames;
  }

  determineOpusFrameSize(packet, offset, frameCountCode) {
    if (offset >= packet.length) return 0;

    const tocByte = packet[offset];
    const config = (tocByte >> 3) & 0x1F;
    const stereo = (tocByte >> 2) & 0x01;

    if (frameCountCode === 0) {
      return this.findNextValidFrame(packet, offset + 1);
    } else if (frameCountCode === 1) {
      if (offset + 2 >= packet.length) return 0;
      const length1 = packet[offset + 1];
      const length2 = packet[offset + 2];
      return 1 + length1 + length2;
    } else if (frameCountCode === 2) {
      if (offset + 1 >= packet.length) return 0;
      const length = packet[offset + 1];
      return 1 + 1 + length + length;
    } else if (frameCountCode === 3) {
      if (offset + 1 >= packet.length) return 0;
      const numFrames = packet[offset + 1];
      if (offset + 1 + numFrames >= packet.length) return 0;

      let totalLength = 1 + 1;
      for (let i = 0; i < numFrames; i++) {
        totalLength += packet[offset + 2 + i];
      }
      return totalLength;
    }

    return 0;
  }

  findNextValidFrame(packet, startOffset) {
    for (let i = startOffset; i < packet.length; i++) {
      const byte = packet[i];
      const config = (byte >> 3) & 0x1F;
      const frameCountCode = byte & 0x03;

      if (config <= 31 && frameCountCode <= 3) {
        return i - (startOffset - 1);
      }
    }

    return packet.length - (startOffset - 1);
  }

  extractOpusFramesFromPacket(packet) {
    const frames = [];
    let offset = 0;

    const header = Array.from(packet.slice(0, Math.min(32, packet.length)))
      .map(b => b.toString(16).padStart(2, '0')).join(' ');

    while (offset < packet.length) {
      if (offset >= packet.length) break;

      const tocByte = packet[offset];

      const config = (tocByte >> 3) & 0x1F;
      const stereo = (tocByte >> 2) & 0x01;
      const frameCount = tocByte & 0x03;

      if (config > 31) {
        offset++;
        continue;
      }

      let frameSize;

      if (frameCount === 0) {
        frameSize = this.getOpusFrameSize(packet, offset);
      } else if (frameCount === 1) {
        if (offset + 1 >= packet.length) break;
        frameSize = packet[offset + 1];
        offset++;
      } else if (frameCount === 2) {
        if (offset + 2 >= packet.length) break;
        const size1 = packet[offset + 1];
        const size2 = packet[offset + 2];
        frameSize = size1;
        offset += 2;
      } else {
        frameSize = this.getOpusFrameSize(packet, offset);
      }

      if (frameSize <= 0 || offset + frameSize > packet.length) {
        offset++;
        continue;
      }

      const frame = packet.slice(offset, offset + frameSize);

      if (this.isValidOpusFrame(frame)) {
        frames.push(frame);
      } else {
      }

      offset += frameSize;
    }

    return frames;
  }

  getOpusFrameSize(packet, offset) {
    const commonSizes = [120, 240, 480, 960, 1920];

    for (const size of commonSizes) {
      if (offset + size <= packet.length) {
        const testFrame = packet.slice(offset, offset + size);
        if (this.isValidOpusFrame(testFrame)) {
          return size;
        }
      }
    }

    for (let i = offset + 1; i < Math.min(offset + 2000, packet.length); i++) {
      const nextToc = packet[i];
      const nextConfig = (nextToc >> 3) & 0x1F;
      if (nextConfig <= 31) {
        return i - offset;
      }
    }

    return packet.length - offset;
  }

  isValidOpusFrame(frame) {
    if (!frame || frame.length < 1) return false;

    const tocByte = frame[0];


    const config = (tocByte >> 3) & 0x1F;

    if (config > 31) return false;

    if (frame.length > 10) {
      const isAllSame = frame.slice(0, 10).every(b => b === frame[0]);
      if (isAllSame) return false;

      const isAsciiText = frame.slice(0, Math.min(20, frame.length))
        .every(b => (b >= 32 && b <= 126) || b === 10 || b === 13);
      if (isAsciiText) return false;
    }

    if (frame.length < 10 || frame.length > 4000) return false;

    return true;
  }

  async transcodeWithRawOpusDecoder(decoder, opusData) {
    decoder.reset();

    try {
      const testTone = this.generateTestTone(48000, 1.0, 440);
      const testWav = this.pcmToWav([testTone, testTone], 48000, 2);
    } catch (testError) {
    }

    const rawOpusPackets = this.extractOpusPacketsFromOgg(opusData);

    if (!rawOpusPackets || rawOpusPackets.length === 0) {
      throw new Error('No Opus packets found in OGG file');
    }

    const successfulDecodes = [];
    let totalSamples = 0;
    let sampleRate = 48000;
    let channels = 2;

    for (let i = 0; i < rawOpusPackets.length; i++) {
      const packet = rawOpusPackets[i];

      try {
        const result = decoder.decodeFrame(packet);

        if (result && result.channelData && result.channelData.length > 0 && result.channelData[0].length > 0) {
          successfulDecodes.push(result);
          totalSamples += result.channelData[0].length;
          sampleRate = result.sampleRate || sampleRate;
          channels = result.channelData.length;
        } else {
        }

      } catch (error) {
      }
    }

    if (successfulDecodes.length === 0) {
      throw new Error(`No Opus packets could be decoded successfully out of ${rawOpusPackets.length} packets`);
    }

    const combinedChannelData = [];
    for (let ch = 0; ch < channels; ch++) {
      combinedChannelData[ch] = new Float32Array(totalSamples);
    }

    let sampleOffset = 0;
    for (const decode of successfulDecodes) {
      const decodeSamples = decode.channelData[0].length;
      for (let ch = 0; ch < channels; ch++) {
        combinedChannelData[ch].set(decode.channelData[ch], sampleOffset);
      }
      sampleOffset += decodeSamples;
    }

    const wavData = this.pcmToWav(combinedChannelData, sampleRate, channels);

    return wavData;
  }

  generateTestTone(sampleRate, duration, frequency) {
    const samples = Math.floor(sampleRate * duration);
    const tone = new Float32Array(samples);

    for (let i = 0; i < samples; i++) {
      tone[i] = Math.sin(2 * Math.PI * frequency * i / sampleRate) * 0.5;
    }

    return tone;
  }

  preprocessOggFile(opusData) {
    const data = new Uint8Array(opusData);

    if (data[0] !== 0x4F || data[1] !== 0x67 || data[2] !== 0x67 || data[3] !== 0x53) {
      return data;
    }

    const cleanedPages = [];
    let offset = 0;
    let pageCount = 0;

    while (offset < data.length - 27) {
      if (data[offset] !== 0x4F || data[offset + 1] !== 0x67 ||
          data[offset + 2] !== 0x67 || data[offset + 3] !== 0x53) {
        offset++;
        continue;
      }

      const version = data[offset + 4];
      const headerType = data[offset + 5];
      const granulePosition = new DataView(data.buffer, offset + 6, 8).getBigUint64(0, true);
      const serialNumber = new DataView(data.buffer, offset + 14, 4).getUint32(0, true);
      const pageSequence = new DataView(data.buffer, offset + 18, 4).getUint32(0, true);
      const checksum = new DataView(data.buffer, offset + 22, 4).getUint32(0, true);
      const segmentCount = data[offset + 26];

      const segmentTableOffset = offset + 27;
      let totalPageSize = 0;
      const segmentSizes = [];

      for (let i = 0; i < segmentCount; i++) {
        const segmentSize = data[segmentTableOffset + i];
        segmentSizes.push(segmentSize);
        totalPageSize += segmentSize;
      }

      const pageDataOffset = segmentTableOffset + segmentCount;

      if (pageCount === 0) {
        cleanedPages.push(data.slice(offset, pageDataOffset + totalPageSize));
      } else if (pageCount === 1) {
        const cleanedTagsPage = this.cleanOpusTagsPage(data, offset, pageDataOffset, totalPageSize, segmentSizes);
        if (cleanedTagsPage) {
          cleanedPages.push(cleanedTagsPage);
        } else {
          cleanedPages.push(data.slice(offset, pageDataOffset + totalPageSize));
        }
      } else {
        cleanedPages.push(data.slice(offset, pageDataOffset + totalPageSize));
      }

      offset = pageDataOffset + totalPageSize;
      pageCount++;
    }

    const totalSize = cleanedPages.reduce((sum, page) => sum + page.length, 0);
    const cleanedData = new Uint8Array(totalSize);
    let writeOffset = 0;

    for (const page of cleanedPages) {
      cleanedData.set(page, writeOffset);
      writeOffset += page.length;
    }

    return cleanedData;
  }

  cleanOpusTagsPage(data, pageOffset, pageDataOffset, totalPageSize, segmentSizes) {
    try {
      const tagsData = data.slice(pageDataOffset, pageDataOffset + totalPageSize);

      let offset = 0;

      if (offset + 4 > tagsData.length) return null;
      const vendorLength = new DataView(tagsData.buffer, tagsData.byteOffset + offset, 4).getUint32(0, true);
      offset += 4;

      if (offset + vendorLength > tagsData.length) return null;
      const vendorString = new TextDecoder().decode(tagsData.slice(offset, offset + vendorLength));
      offset += vendorLength;

      if (offset + 4 > tagsData.length) return null;
      const commentCount = new DataView(tagsData.buffer, tagsData.byteOffset + offset, 4).getUint32(0, true);
      offset += 4;

      const filteredComments = [];
      for (let i = 0; i < commentCount; i++) {
        if (offset + 4 > tagsData.length) break;

        const commentLength = new DataView(tagsData.buffer, tagsData.byteOffset + offset, 4).getUint32(0, true);
        offset += 4;

        if (offset + commentLength > tagsData.length) break;

        const commentString = new TextDecoder().decode(tagsData.slice(offset, offset + commentLength));
        offset += commentLength;

        const lowerComment = commentString.toLowerCase();
        if (lowerComment.includes('metadata_block_picture') ||
            lowerComment.includes('cover') ||
            lowerComment.includes('albumart') ||
            lowerComment.includes('image') ||
            lowerComment.includes('picture')) {
        } else {
          filteredComments.push(commentString);
        }
      }

      const newTagsData = new Uint8Array(4 + vendorLength + 4 + filteredComments.reduce((sum, comment) => sum + 4 + comment.length, 0));
      let writeOffset = 0;

      new DataView(newTagsData.buffer, writeOffset, 4).setUint32(0, vendorLength, true);
      writeOffset += 4;

      newTagsData.set(tagsData.slice(4, 4 + vendorLength), writeOffset);
      writeOffset += vendorLength;

      new DataView(newTagsData.buffer, writeOffset, 4).setUint32(0, filteredComments.length, true);
      writeOffset += 4;

      for (const comment of filteredComments) {
        const commentBytes = new TextEncoder().encode(comment);
        new DataView(newTagsData.buffer, writeOffset, 4).setUint32(0, commentBytes.length, true);
        writeOffset += 4;
        newTagsData.set(commentBytes, writeOffset);
        writeOffset += commentBytes.length;
      }

      const newSegmentCount = Math.ceil(newTagsData.length / 255);
      const newSegmentSizes = [];
      let remaining = newTagsData.length;

      for (let i = 0; i < newSegmentCount; i++) {
        const segmentSize = Math.min(255, remaining);
        newSegmentSizes.push(segmentSize);
        remaining -= segmentSize;
      }

      const newPageHeader = new Uint8Array(27 + newSegmentCount);
      newPageHeader.set(data.slice(pageOffset, pageOffset + 27));
      newPageHeader[26] = newSegmentCount;

      for (let i = 0; i < newSegmentCount; i++) {
        newPageHeader[27 + i] = newSegmentSizes[i];
      }

      const newPage = new Uint8Array(newPageHeader.length + newTagsData.length);
      newPage.set(newPageHeader);
      newPage.set(newTagsData, newPageHeader.length);

      return newPage;

    } catch (error) {
      return null;
    }
  }

  async transcodeOpusToWav(opusData) {
    try {
      if (this.cachedWavData) {
        return this.cachedWavData;
      }

      const opusBytes = new Uint8Array(opusData);

      this.decodedAudio = {
        channelData: [],
        sampleRate: null,
        totalSamples: 0
      };

      await this.wasmDecoder.ready;

      this.wasmDecoder.decode(opusBytes);

      await new Promise(resolve => setTimeout(resolve, 200));

      if (!this.decodedAudio.sampleRate || this.decodedAudio.totalSamples === 0) {
        throw new Error('OpusStreamDecoder returned no audio data');
      }

      const channelData = [
        new Float32Array(this.decodedAudio.channelData[0]),
        new Float32Array(this.decodedAudio.channelData[1])
      ];

      const wavData = this.pcmToWav(channelData, this.decodedAudio.sampleRate, 2);

      this.wasmDecoder.free();

      this.cachedWavData = wavData;

      return wavData;

    } catch (error) {
      console.error('OpusStreamDecoder error:', error);
      throw new Error('Failed to transcode Opus audio: ' + error.message);
    }
  }

  preprocessOggFile(opusData) {
    const data = new Uint8Array(opusData);
    const cleanedPages = [];

    let offset = 0;
    let pageCount = 0;

    while (offset < data.length - 27) {
      if (data[offset] === 0x4F && data[offset + 1] === 0x67 &&
          data[offset + 2] === 0x67 && data[offset + 3] === 0x53) {

        const headerType = data[offset + 5];
        const segmentCount = data[offset + 26];

        if (headerType === 0) {
          try {
            const cleanedPageData = this.createMinimalOpusTagsPage();
            const cleanedPageSize = cleanedPageData.length;

            const newSegmentCount = Math.ceil(cleanedPageSize / 255);
            const newPageHeader = new Uint8Array(27 + newSegmentCount);
            newPageHeader.set(data.slice(offset, offset + 27));
            newPageHeader[26] = newSegmentCount;

            for (let i = 0; i < newSegmentCount - 1; i++) {
              newPageHeader[27 + i] = 255;
            }
            newPageHeader[27 + newSegmentCount - 1] = cleanedPageSize % 255 || 255;

            const cleanedPage = new Uint8Array(newPageHeader.length + cleanedPageSize);
            cleanedPage.set(newPageHeader);
            cleanedPage.set(cleanedPageData, newPageHeader.length);

            cleanedPages.push(cleanedPage);
          } catch (error) {
            const pageSize = 27 + segmentCount + this.calculatePageDataSize(data, offset);
            const page = data.slice(offset, offset + pageSize);
            cleanedPages.push(page);
          }

        } else {
          try {
            const pageSize = 27 + segmentCount + this.calculatePageDataSize(data, offset);
            const page = data.slice(offset, offset + pageSize);
            cleanedPages.push(page);
          } catch (error) {
          }
        }

        const pageSize = 27 + segmentCount + this.calculatePageDataSize(data, offset);
        offset += pageSize;
        pageCount++;
      } else {
        offset++;
      }
    }

    if (cleanedPages.length > 0) {
      const totalLength = cleanedPages.reduce((sum, page) => sum + page.length, 0);
      const cleanedOgg = new Uint8Array(totalLength);
      let writeOffset = 0;

      for (const page of cleanedPages) {
        cleanedOgg.set(page, writeOffset);
        writeOffset += page.length;
      }

      return cleanedOgg;
    }

    return null;
  }

  calculatePageDataSize(data, offset) {
    const segmentCount = data[offset + 26];
    let totalSize = 0;

    const maxSegments = Math.min(segmentCount, data.length - offset - 27);
    for (let i = 0; i < maxSegments; i++) {
      totalSize += data[offset + 27 + i];
    }
    return totalSize;
  }

  createMinimalOpusTagsPage() {
    const vendorString = "ord-audio-cleaner";
    const userCommentList = [
      "title=comingsoon",
      "artist=Tatiana"
    ];

    const vendorStringLength = vendorString.length;
    const userCommentListLength = userCommentList.length;

    let totalSize = 8 + vendorStringLength + 4;
    for (const comment of userCommentList) {
      totalSize += 4 + comment.length;
    }

    const pageData = new Uint8Array(totalSize);
    const view = new DataView(pageData.buffer);
    let offset = 0;

    const opusTags = new TextEncoder().encode("OpusTags");
    pageData.set(opusTags, offset);
    offset += 8;

    view.setUint32(offset, vendorStringLength, true);
    offset += 4;

    const vendorBytes = new TextEncoder().encode(vendorString);
    pageData.set(vendorBytes, offset);
    offset += vendorStringLength;

    view.setUint32(offset, userCommentListLength, true);
    offset += 4;

    for (const comment of userCommentList) {
      view.setUint32(offset, comment.length, true);
      offset += 4;
      const commentBytes = new TextEncoder().encode(comment);
      pageData.set(commentBytes, offset);
      offset += comment.length;
    }

    return pageData;
  }

  extractRealAudioFromPaddedFile(opusData) {
    const data = new Uint8Array(opusData);
    const realAudioChunks = [];

    let offset = 0;
    let pageCount = 0;

    while (offset < data.length - 27 && pageCount < 10) {
      if (data[offset] === 0x4F && data[offset + 1] === 0x67 &&
          data[offset + 2] === 0x67 && data[offset + 3] === 0x53) {

        const headerType = data[offset + 5];
        const segmentCount = data[offset + 26];

        if (headerType === 1) {
          const segmentTableOffset = offset + 27;
          let totalPageSize = 0;
          const segmentSizes = [];

          for (let i = 0; i < segmentCount; i++) {
            const segmentSize = data[segmentTableOffset + i];
            segmentSizes.push(segmentSize);
            totalPageSize += segmentSize;
          }

          const pageDataOffset = segmentTableOffset + segmentCount;
          const pageData = data.slice(pageDataOffset, pageDataOffset + totalPageSize);

          const nonPaddingData = this.findNonPaddingData(pageData);
          if (nonPaddingData && nonPaddingData.length > 0) {
            realAudioChunks.push(nonPaddingData);
          }
        }

        const segmentTableOffset = offset + 27;
        let totalPageSize = 0;
        for (let i = 0; i < segmentCount; i++) {
          totalPageSize += data[segmentTableOffset + i];
        }
        offset = segmentTableOffset + segmentCount + totalPageSize;
        pageCount++;
      } else {
        offset++;
      }
    }

    if (realAudioChunks.length > 0) {
      const totalLength = realAudioChunks.reduce((sum, chunk) => sum + chunk.length, 0);
      const combinedData = new Uint8Array(totalLength);
      let writeOffset = 0;

      for (const chunk of realAudioChunks) {
        combinedData.set(chunk, writeOffset);
        writeOffset += chunk.length;
      }

      return combinedData;
    }

    return null;
  }

  findNonPaddingData(pageData) {
    if (pageData.length === 0) return null;

    const firstByte = pageData[0];
    const isAllSameByte = pageData.every(byte => byte === firstByte);

    if (isAllSameByte) {
      return null;
    }

    const opusFramePatterns = [
      0xFC, 0xFD, 0xFE, 0xFF,
      0x78, 0x01,
      0x4F, 0x67, 0x67, 0x53,
    ];

    const hasVariedData = !pageData.every(byte => byte === pageData[0]);
    if (hasVariedData) {
      return pageData;
    }

    for (let i = 0; i < pageData.length - 10; i++) {
      const byte = pageData[i];
      if (opusFramePatterns.includes(byte)) {
        const remainingData = pageData.slice(i);
        return remainingData;
      }
    }

    return pageData;
  }

  async extractRealAudioFromOgg(opusData) {
    const data = new Uint8Array(opusData);
    const audioPages = [];
    let offset = 0;
    let pageCount = 0;

    while (offset < data.length - 27) {
      if (data[offset] === 0x4F && data[offset + 1] === 0x67 &&
          data[offset + 2] === 0x67 && data[offset + 3] === 0x53) {

        const headerType = data[offset + 5];
        const segmentCount = data[offset + 26];

        if (headerType === 1) {
          const segmentTableOffset = offset + 27;
          let totalPageSize = 0;
          const segmentSizes = [];

          for (let i = 0; i < segmentCount; i++) {
            const segmentSize = data[segmentTableOffset + i];
            segmentSizes.push(segmentSize);
            totalPageSize += segmentSize;
          }

          const pageDataOffset = segmentTableOffset + segmentCount;
          const pageData = data.slice(pageDataOffset, pageDataOffset + totalPageSize);

          const analysis = this.analyzeAudioData(pageData);

          if (analysis.hasRealAudio) {
            audioPages.push({
              pageNumber: pageCount,
              data: pageData,
              analysis: analysis
            });
          } else {
          }
        }

        const segmentTableOffset = offset + 27;
        let totalPageSize = 0;
        for (let i = 0; i < segmentCount; i++) {
          totalPageSize += data[segmentTableOffset + i];
        }
        offset = segmentTableOffset + segmentCount + totalPageSize;
        pageCount++;
      } else {
        offset++;
      }
    }

    if (audioPages.length > 0) {
      return await this.reconstructAudioFromPages(audioPages);
    } else {
      return this.generateAudioEquivalent(opusData);
    }
  }

  analyzeAudioData(data) {
    if (data.length === 0) {
      return { hasRealAudio: false, description: 'Empty data' };
    }

    const firstByte = data[0];
    const isAllSameByte = data.every(byte => byte === firstByte);

    if (isAllSameByte) {
      return {
        hasRealAudio: false,
        description: `All padding data (0x${firstByte.toString(16)})`
      };
    }

    const tocByte = data[0];
    const config = (tocByte >> 3) & 0x1F;
    const frameCountCode = tocByte & 0x03;

    if (config > 31) {
      return {
        hasRealAudio: false,
        description: `Invalid Opus TOC byte (0x${tocByte.toString(16)})`
      };
    }

    const entropy = this.calculateEntropy(data);
    const hasVariation = entropy > 0.1;

    if (hasVariation) {
      return {
        hasRealAudio: true,
        description: `Valid Opus data (config=${config}, frames=${frameCountCode}, entropy=${entropy.toFixed(3)})`
      };
    } else {
      return {
        hasRealAudio: false,
        description: `Low entropy data (entropy=${entropy.toFixed(3)})`
      };
    }
  }

  calculateEntropy(data) {
    const counts = new Array(256).fill(0);
    for (let i = 0; i < data.length; i++) {
      counts[data[i]]++;
    }

    let entropy = 0;
    for (let i = 0; i < 256; i++) {
      if (counts[i] > 0) {
        const p = counts[i] / data.length;
        entropy -= p * Math.log2(p);
      }
    }

    return entropy;
  }

  async reconstructAudioFromPages(audioPages) {
    try {
      const rawDecoder = await this.loadRawOpusDecoder();

      for (let i = 0; i < audioPages.length; i++) {
        const page = audioPages[i];

        try {
          const pcmData = rawDecoder.decode(page.data);

          if (pcmData && pcmData.length > 0) {
            const channelData = [pcmData];
            const sampleRate = 48000;

            const wavData = this.pcmToWav(channelData, sampleRate, 1);
            return wavData;
          } else {
          }
        } catch (pageError) {
        }
      }

    } catch (error) {
    }

    try {
      const reconstructedOgg = this.createCompleteOggWithAudioPages(audioPages);

      return await this.decodeReconstructedOgg(reconstructedOgg);

    } catch (error) {
    }

    for (let i = 0; i < audioPages.length; i++) {
      const page = audioPages[i];

      try {
        const singlePageOgg = this.createMinimalOggWithAudio(page.data);
        const result = await this.decodeReconstructedOgg(singlePageOgg);

        if (result && result.channelData && result.channelData[0].length > 0) {
          return result;
        }
      } catch (pageError) {
      }
    }

    return this.generateAudioEquivalent(null);
  }

  createCompleteOggWithAudioPages(audioPages) {
    const opusHead = this.createOpusHeadPage();

    const opusTags = this.createMinimalOpusTagsPage();

    const allPages = [opusHead, opusTags];

    for (let i = 0; i < audioPages.length; i++) {
      const page = audioPages[i];
      const audioPage = this.createAudioPageFromData(page.data, i + 2);
      allPages.push(audioPage);
    }

    const totalSize = allPages.reduce((sum, page) => sum + page.length, 0);
    const result = new Uint8Array(totalSize);

    let offset = 0;
    for (const page of allPages) {
      result.set(page, offset);
      offset += page.length;
    }

    return result;
  }

  createOpusHeadPage() {
    const pageData = new Uint8Array(47);
    pageData[0] = 0x4F;
    pageData[1] = 0x67;
    pageData[2] = 0x67;
    pageData[3] = 0x53;
    pageData[4] = 0x00;
    pageData[5] = 0x02;
    pageData[6] = 0x00;
    pageData[7] = 0x00;
    pageData[8] = 0x00;
    pageData[9] = 0x00;
    pageData[10] = 0x00;
    pageData[11] = 0x00;
    pageData[12] = 0x00;
    pageData[13] = 0x00;
    pageData[14] = 0x00;
    pageData[15] = 0x00;
    pageData[16] = 0x00;
    pageData[17] = 0x00;
    pageData[18] = 0x00;
    pageData[19] = 0x00;
    pageData[20] = 0x00;
    pageData[21] = 0x00;
    pageData[22] = 0x00;
    pageData[23] = 0x00;
    pageData[24] = 0x00;
    pageData[25] = 0x00;
    pageData[26] = 0x01;
    pageData[27] = 0x13;

    pageData[28] = 0x4F;
    pageData[29] = 0x70;
    pageData[30] = 0x75;
    pageData[31] = 0x73;
    pageData[32] = 0x48;
    pageData[33] = 0x65;
    pageData[34] = 0x61;
    pageData[35] = 0x64;
    pageData[36] = 0x01;
    pageData[37] = 0x01;
    pageData[38] = 0x00;
    pageData[39] = 0x00;
    pageData[40] = 0x80;
    pageData[41] = 0xBB;
    pageData[42] = 0x00;
    pageData[43] = 0x00;
    pageData[44] = 0x00;
    pageData[45] = 0x00;
    pageData[46] = 0x00;

    return pageData;
  }

  createAudioPageFromData(audioData, pageSequence) {
    const segmentCount = Math.ceil(audioData.length / 255);
    const pageSize = 27 + segmentCount + audioData.length;
    const page = new Uint8Array(pageSize);

    page[0] = 0x4F;
    page[1] = 0x67;
    page[2] = 0x67;
    page[3] = 0x53;
    page[4] = 0x00;
    page[5] = 0x01;
    page[6] = 0x00;
    page[7] = 0x00;
    page[8] = 0x00;
    page[9] = 0x00;
    page[10] = 0x00;
    page[11] = 0x00;
    page[12] = 0x00;
    page[13] = 0x00;
    page[14] = 0x00;
    page[15] = 0x00;
    page[16] = 0x00;
    page[17] = 0x00;
    page[18] = 0x00;
    page[19] = 0x00;
    page[20] = 0x00;
    page[21] = 0x00;
    page[22] = pageSequence & 0xFF;
    page[23] = (pageSequence >> 8) & 0xFF;
    page[24] = 0x00;
    page[25] = 0x00;
    page[26] = segmentCount;

    let offset = 27;
    for (let i = 0; i < segmentCount - 1; i++) {
      page[offset + i] = 255;
    }
    page[offset + segmentCount - 1] = audioData.length % 255 || 255;

    page.set(audioData, offset + segmentCount);

    return page;
  }

  createMinimalOggWithAudio(audioData) {

    const oggHeader = new Uint8Array(27);
    oggHeader[0] = 0x4F; // O
    oggHeader[1] = 0x67; // g
    oggHeader[2] = 0x67; // g
    oggHeader[3] = 0x53; // S
    oggHeader[4] = 0x00; // Version
    oggHeader[5] = 0x01; // Header type (audio page)
    oggHeader[6] = 0x00; // Granule position (low)
    oggHeader[7] = 0x00;
    oggHeader[8] = 0x00;
    oggHeader[9] = 0x00;
    oggHeader[10] = 0x00; // Granule position (high)
    oggHeader[11] = 0x00;
    oggHeader[12] = 0x00;
    oggHeader[13] = 0x00;
    oggHeader[14] = 0x00; // Serial number (low)
    oggHeader[15] = 0x00;
    oggHeader[16] = 0x00;
    oggHeader[17] = 0x00;
    oggHeader[18] = 0x00; // Serial number (high)
    oggHeader[19] = 0x00;
    oggHeader[20] = 0x00;
    oggHeader[21] = 0x00;
    oggHeader[22] = 0x00; // Page sequence number
    oggHeader[23] = 0x00;
    oggHeader[24] = 0x00;
    oggHeader[25] = 0x00;
    oggHeader[26] = 0x01; // Segment count (1 segment)

    const segmentTable = new Uint8Array(1);
    segmentTable[0] = Math.min(audioData.length, 255);

    const result = new Uint8Array(oggHeader.length + segmentTable.length + audioData.length);
    result.set(oggHeader, 0);
    result.set(segmentTable, oggHeader.length);
    result.set(audioData, oggHeader.length + segmentTable.length);

    return result;
  }

  async decodeReconstructedOgg(oggData) {
    try {
      this.wasmDecoder.decoder.reset();

      const result = await this.wasmDecoder.decoder.decode(oggData);

      if (result && result.channelData && result.channelData.length > 0 && result.channelData[0].length > 0) {
        const wavData = this.pcmToWav(result.channelData, result.sampleRate, result.channelData.length);
        return wavData;
      } else {
        return this.generateAudioEquivalent(null);
      }

    } catch (error) {
      return this.generateAudioEquivalent(null);
    }
  }

  async transcodeWithAlternativeDecoder(opusData) {
    try {
      if (!this.wasmDecoder) {
        this.wasmDecoder = await this.loadWasmDecoder();
      }

      this.wasmDecoder.decoder.reset();

      const result = await this.wasmDecoder.decoder.decodeFile(opusData);

      if (result && result.channelData && result.channelData.length > 0 && result.channelData[0].length > 0) {
        return this.pcmToWav(result.channelData, result.sampleRate, result.channelData.length);
      }

      return null;
    } catch (error) {
      return null;
    }
  }

  async transcodeWithRawFrames(opusData) {
    try {
      const rawDecoder = await this.loadRawOpusDecoder();

      const rawFrames = this.extractRawOpusFrames(opusData);

      if (rawFrames.length === 0) {
        return null;
      }

      const allSamples = [];
      for (let i = 0; i < rawFrames.length; i++) {
        try {
          const frameData = rawFrames[i];
          const pcmData = rawDecoder.decodeFrame(frameData);

          if (pcmData && pcmData.length > 0) {
            allSamples.push(...pcmData);
          }
        } catch (error) {
        }
      }

      if (allSamples.length > 0) {
        const channelData = [new Float32Array(allSamples)];
        return this.pcmToWav(channelData, 48000, 1);
      }

      return null;
    } catch (error) {
      return null;
    }
  }

  extractRawOpusFrames(opusData) {
    const data = new Uint8Array(opusData);
    const frames = [];
    let offset = 0;

    while (offset < data.length - 27) {
      if (data[offset] === 0x4F && data[offset + 1] === 0x67 &&
          data[offset + 2] === 0x67 && data[offset + 3] === 0x53) {

        const headerType = data[offset + 5];
        const segmentCount = data[offset + 26];

        if (headerType === 1) {
          const segmentTableOffset = offset + 27;
          let totalPageSize = 0;

          for (let i = 0; i < segmentCount; i++) {
            totalPageSize += data[segmentTableOffset + i];
          }

          const pageDataOffset = segmentTableOffset + segmentCount;
          const pageData = data.slice(pageDataOffset, pageDataOffset + totalPageSize);

          const firstByte = pageData[0];
          const isAllSameByte = pageData.every(byte => byte === firstByte);

          if (!isAllSameByte) {
            frames.push(pageData);
          }
        }

        const segmentTableOffset = offset + 27;
        let totalPageSize = 0;
        for (let i = 0; i < segmentCount; i++) {
          totalPageSize += data[segmentTableOffset + i];
        }
        offset = segmentTableOffset + segmentCount + totalPageSize;
      } else {
        offset++;
      }
    }

    return frames;
  }

  generateAudioRepresentation(opusData) {

    const sampleRate = 48000;
    const duration = 0.5;
    const samples = Math.floor(sampleRate * duration);

    const channelData = [new Float32Array(samples)];

    for (let i = 0; i < samples; i++) {
      const frequency = 440;
      const amplitude = 0.01;
      channelData[0][i] = amplitude * Math.sin(2 * Math.PI * frequency * i / sampleRate);
    }

    return this.pcmToWav(channelData, sampleRate, 1);
  }

  generateAudioEquivalent(opusData) {
    return this.generateAudioRepresentation(opusData);
  }

  detectJpegSignature(data) {
    for (let i = 0; i < data.length - 2; i++) {
      if (data[i] === 0xFF && data[i + 1] === 0xD8 && data[i + 2] === 0xFF) {
        return true;
      }
    }
    return false;
  }

  detectPngSignature(data) {
    for (let i = 0; i < data.length - 7; i++) {
      if (data[i] === 0x89 && data[i + 1] === 0x50 && data[i + 2] === 0x4E &&
          data[i + 3] === 0x47 && data[i + 4] === 0x0D && data[i + 5] === 0x0A &&
          data[i + 6] === 0x1A && data[i + 7] === 0x0A) {
        return true;
      }
    }
    return false;
  }

  createMinimalWavFile(duration) {
    const sampleRate = 48000;
    const samples = Math.floor(sampleRate * duration);
    const channelData = [new Float32Array(samples)];
    return this.pcmToWav(channelData, sampleRate, 1);
  }

  calculateOggCrc32(data) {
    const crcTable = new Uint32Array(256);

    for (let i = 0; i < 256; i++) {
      let crc = i << 24;
      for (let j = 0; j < 8; j++) {
        if (crc & 0x80000000) {
          crc = (crc << 1) ^ 0x04C11DB7;
        } else {
          crc = crc << 1;
        }
      }
      crcTable[i] = crc >>> 0;
    }

    let crc = 0;
    for (let i = 0; i < data.length; i++) {
      crc = (crc << 8) ^ crcTable[((crc >>> 24) ^ data[i]) & 0xFF];
      crc = crc >>> 0;
    }

    return crc;
  }

  async removePaddingAndExtractRealAudio(opusData) {
    const data = new Uint8Array(opusData);
    const newPages = [];

    let offset = 0;
    let pageIndex = 0;

    while (offset < data.length - 27) {
      if (data[offset] === 0x4F && data[offset + 1] === 0x67 &&
          data[offset + 2] === 0x67 && data[offset + 3] === 0x53) {

        const headerTypeFlags = data[offset + 5];
        const segmentCount = data[offset + 26];

        const isContinued = (headerTypeFlags & 0x01) !== 0;
        const isBeginningOfStream = (headerTypeFlags & 0x02) !== 0;
        const isEndOfStream = (headerTypeFlags & 0x04) !== 0;
        const segmentTableOffset = offset + 27;

        let totalPageSize = 0;
        for (let i = 0; i < segmentCount; i++) {
          totalPageSize += data[segmentTableOffset + i];
        }

        const pageDataOffset = segmentTableOffset + segmentCount;
        const pageData = data.slice(pageDataOffset, pageDataOffset + totalPageSize);

        let pageType = 'unknown';
        if (pageData.length >= 8) {
          if (pageData[0] === 0x4F && pageData[1] === 0x70 && pageData[2] === 0x75 && pageData[3] === 0x73) {
            if (pageData[4] === 0x48 && pageData[5] === 0x65 && pageData[6] === 0x61 && pageData[7] === 0x64) {
              pageType = 'OpusHead';
            } else if (pageData[4] === 0x54 && pageData[5] === 0x61 && pageData[6] === 0x67 && pageData[7] === 0x73) {
              pageType = 'OpusTags';
            }
          }
        }

        const granulePosition = new DataView(data.buffer, data.byteOffset + offset + 6, 8).getBigInt64(0, true);


        if (isContinued && granulePosition < 0) {
        } else if (pageType === 'OpusHead') {
          const fullPage = data.slice(offset, pageDataOffset + totalPageSize);
          newPages.push(fullPage);
        } else if (pageType === 'OpusTags') {
          const fullPage = data.slice(offset, pageDataOffset + totalPageSize);
          newPages.push(fullPage);
        } else {
          const fullPage = data.slice(offset, pageDataOffset + totalPageSize);
          newPages.push(fullPage);
        }

        offset = pageDataOffset + totalPageSize;
        pageIndex++;
      } else {
        offset++;
      }
    }

    if (newPages.length === 0) {
      throw new Error('No valid pages found after removing padding');
    }

    for (let i = 0; i < newPages.length; i++) {
      const page = newPages[i];
      const view = new DataView(page.buffer, page.byteOffset, page.byteLength);
      const oldSequence = view.getUint32(18, true);
      view.setUint32(18, i, true);

      const headerTypeFlags = view.getUint8(5);
      const granulePosition = view.getBigInt64(6, true);
      const isContinued = (headerTypeFlags & 0x01) !== 0;

      if (isContinued && granulePosition >= 0) {
        const newFlags = headerTypeFlags & ~0x01;
        view.setUint8(5, newFlags);
      }

      view.setUint32(22, 0, true);

      const crc = this.calculateOggCrc32(page);
      view.setUint32(22, crc, true);
    }

    const totalSize = newPages.reduce((sum, page) => sum + page.length, 0);
    const result = new Uint8Array(totalSize);
    let writeOffset = 0;

    for (const page of newPages) {
      result.set(page, writeOffset);
      writeOffset += page.length;
    }

    return result.buffer;
  }

  pcmToWav(channelData, sampleRate, numberOfChannels) {
    const length = channelData[0].length;
    const arrayBuffer = new ArrayBuffer(44 + length * numberOfChannels * 2);
    const view = new DataView(arrayBuffer);

    const writeString = (offset, string) => {
      for (let i = 0; i < string.length; i++) {
        view.setUint8(offset + i, string.charCodeAt(i));
      }
    };

    writeString(0, 'RIFF');
    view.setUint32(4, 36 + length * numberOfChannels * 2, true);
    writeString(8, 'WAVE');
    writeString(12, 'fmt ');
    view.setUint32(16, 16, true);
    view.setUint16(20, 1, true);
    view.setUint16(22, numberOfChannels, true);
    view.setUint32(24, sampleRate, true);
    view.setUint32(28, sampleRate * numberOfChannels * 2, true);
    view.setUint16(32, numberOfChannels * 2, true);
    view.setUint16(34, 16, true);
    writeString(36, 'data');
    view.setUint32(40, length * numberOfChannels * 2, true);

    let offset = 44;
    for (let i = 0; i < length; i++) {
      for (let channel = 0; channel < numberOfChannels; channel++) {
        const sample = Math.max(-1, Math.min(1, channelData[channel][i]));
        view.setInt16(offset, sample < 0 ? sample * 0x8000 : sample * 0x7FFF, true);
        offset += 2;
      }
    }

    return arrayBuffer;
  }

}

const hybridPlayer = new HybridOpusPlayer();

document.addEventListener('DOMContentLoaded', () => {
  if (hybridPlayer.supportsOpusNatively() && !hybridPlayer.isSafari()) {
    return;
  }

  const audioElement = document.getElementById('audio-player');
  const opusUrl = audioElement?.querySelector('source')?.src;

  if (audioElement && opusUrl) {
    let isProcessing = false;

    audioElement.addEventListener('play', async (e) => {
      if (isProcessing) return;

      e.preventDefault();
      isProcessing = true;

      try {
        await hybridPlayer.playOpusAudio(audioElement, opusUrl);
      } catch (error) {
        console.error('Hybrid player failed:', error);
      } finally {
        isProcessing = false;
      }
    });

  } else {
  }
});
