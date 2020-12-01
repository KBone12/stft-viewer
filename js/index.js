import("../pkg/index.js").then(rust => {
  const WindowFunction = rust.WindowFunction;
  const FourierViewer = rust.FourierViewer;

  const windowFunctionElement = document.getElementById("window_function");
  const fftSizeElement = document.getElementById("fft_size");

  let viewer = null;

  document.getElementById("file").addEventListener("change", function () {
    const files = this.files;
    if (files.length === 0) {
      return;
    }
    const audioContext = new (window.AudioContext || window.webkitAudioContext)();
    files[0].arrayBuffer().then(buffer => audioContext.decodeAudioData(buffer)).then(data => {
      viewer = new FourierViewer(data.getChannelData(0), data.sampleRate);
      document.getElementById("fft_size").max = Math.max(Math.min(data.length, 1 << 16), 2);
      document.getElementById("calculate").click();
    }).catch(console.error).finally(() => {
      audioContext.close();
    });
  }, false);

  function windowNameToWindowFunction(name) {
    if (name == "Blackman") {
      return WindowFunction.Blackman;
    } else if (name == "Hamming") {
      return WindowFunction.Hamming;
    } else if (name == "Hann") {
      return WindowFunction.Hann;
    } else if (name == "Rectangle") {
      return WindowFunction.Rectangle;
    } else {
      console.error("Illegal window function");
      return null;
    }
  }

  document.getElementById("calculate").addEventListener("click", () => {
    if (viewer === null) {
      return;
    }
    const canvas = document.getElementById("canvas");
    const peak_frequencies_element = document.getElementById("peak_frequencies");
    const peak_phases_element = document.getElementById("peak_phases");

    canvas.getContext("2d").clearRect(0, 0, canvas.width, canvas.height);
    peak_frequencies_element.innerHTML = "Peak frequencies:&nbsp;";
    peak_phases_element.innerHTML = "Peak phases [deg]:&nbsp;";

    let fftSize = parseInt(fftSizeElement.value);
    let windowFunction = windowNameToWindowFunction(windowFunctionElement.value);
    viewer.run_fft(fftSize, windowFunction);
    const peak_frequencies = viewer.peak_frequencies(5);
    const peak_phases = viewer.peak_phases(5);
    for (let i = 0; i < peak_frequencies.length; i += 1) {
      peak_frequencies_element.innerHTML += peak_frequencies[i].toFixed(1);
      peak_phases_element.innerHTML += (peak_phases[i] / (2.0 * Math.PI) * 360.0).toFixed(1);
      if (i < peak_frequencies.length - 1) {
        peak_frequencies_element.innerHTML += ",&nbsp;";
        peak_phases_element.innerHTML += ",&nbsp;";
      }
    }
    viewer.draw(canvas);
  });
}).catch(console.error);
