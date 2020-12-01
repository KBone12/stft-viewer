import("../pkg/index.js").then(rust => {
  const WindowFunction = rust.WindowFunction;

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

  self.addEventListener("message", event => {
    const size = event.data.size;
    const windowFunction = windowNameToWindowFunction(event.data.windowFunctionName);

    const spectra = rust.run_stft(event.data.audioData, size, windowFunction);

    const data = {
      spectra: spectra,
      size: size,
    };
    self.postMessage(data, [data.spectra.buffer]);
  });
});
