import { Simulation } from 'mywasm';

const width = 600;
const height = 300;
const canvasScale = 3;

const s = new Simulation(width, height);

enum PostprocField{
  Temperature,
  Phi
}
let postprocField = PostprocField.Phi;

const canvas = <HTMLCanvasElement>document.getElementById("postproc-area");
canvas.width = canvasScale * width;
canvas.height = canvasScale * height;
const context = canvas.getContext("2d");

const toggleFieldButton = <HTMLButtonElement>document.getElementById("toggle-field");
toggleFieldButton.addEventListener("click", () => {
  if(postprocField == PostprocField.Temperature){
    postprocField = PostprocField.Phi;
  } else {
    postprocField = PostprocField.Temperature;
  }
});

const resetButton = <HTMLButtonElement>document.getElementById("reset-button");
resetButton.addEventListener("click", () => {
  s.reset();
});

const kappaValue = <HTMLSpanElement>document.getElementById("kappa-value");
kappaValue.innerText = s.kappa.toPrecision(2).toString();
const kappaSlider = <HTMLInputElement>document.getElementById("kappa-slider");
kappaSlider.addEventListener("input", (event) => {
  console.log((event.target as HTMLInputElement).value);
  const kappaValue = <HTMLSpanElement>document.getElementById("kappa-value");
  kappaValue.innerText = (event.target as HTMLInputElement).value;
});

const deltaValue = <HTMLSpanElement>document.getElementById("delta-value");
deltaValue.innerText = s.delta.toPrecision(2).toString();
const deltaSlider = <HTMLInputElement>document.getElementById("delta-slider");
deltaSlider.addEventListener("input", (event) => {
  console.log((event.target as HTMLInputElement).value);
  const deltaValue = <HTMLSpanElement>document.getElementById("delta-value");
  deltaValue.innerText = (event.target as HTMLInputElement).value;
});


const postprocess = async () => {
  console.time("Simulation");
  s.step();
  console.timeEnd("Simulation");
  let rgbBuffer: Uint8Array = null;
  if(postprocField == PostprocField.Phi){
    rgbBuffer = s.get_phi_rgb();
  } else {
    rgbBuffer = s.get_temperature_rgb();
  }
  const rgbDataArray = new Uint8ClampedArray(rgbBuffer);
  const imageData = new ImageData(rgbDataArray, width, height);
  const bitmap = await createImageBitmap(imageData);
  context.drawImage(bitmap, 0, 0, 3 * width, 3 * height);
  requestAnimationFrame(postprocess);
}

requestAnimationFrame(postprocess);
