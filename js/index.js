import Worker from "./worker.js";

import("../pkg/index.js").then(rust => {

  const WindowFunction = rust.WindowFunction;
  const worker = new Worker();

  const windowFunctionElement = document.getElementById("window_function");
  const fftSizeElement = document.getElementById("fft_size");

  let audioData = null;
  let sampleRate = null;

  document.getElementById("file").addEventListener("change", function () {
    const files = this.files;
    if (files.length === 0) {
      return;
    }
    const audioContext = new (window.AudioContext || window.webkitAudioContext)();
    files[0].arrayBuffer().then(buffer => audioContext.decodeAudioData(buffer)).then(data => {
      audioData = data.getChannelData(0);
      sampleRate = data.sampleRate;
      document.getElementById("fft_size").max = Math.max(Math.min(data.length, 1 << 16), 2);
      document.getElementById("calculate").click();
    }).catch(console.error).finally(() => {
      audioContext.close();
    });
  }, false);

  document.getElementById("calculate").addEventListener("click", () => {
    if (audioData === null) {
      return;
    }
    const canvas = document.getElementById("canvas");

    canvas.getContext("2d").clearRect(0, 0, canvas.width, canvas.height);

    worker.addEventListener("message", event => {
      const spectra = event.data.spectra;
      const size = event.data.size;

      document.getElementById("status").innerText = "Drawing...";
      rust.draw(canvas, spectra, size, sampleRate);

      document.getElementById("status").innerText = "Done";
    }, { once: true });

    document.getElementById("status").innerText = "Calculating...";
    const data = {
      audioData: audioData,
      size: parseInt(fftSizeElement.value),
      windowFunctionName: windowFunctionElement.value,
    };
    // worker.postMessage(data, [data.audioData.buffer]);
    // audioData = null;
    worker.postMessage(data);
  });
}).catch(console.error);
