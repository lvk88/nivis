import { Simulation } from 'mywasm';

enum PostprocField{
  Temperature,
  Phi
}

let postprocField = PostprocField.Temperature;

const width = 300;
const height = 300;

const canvasScale = 3;

const canvas = <HTMLCanvasElement>document.getElementById("postproc-area");
canvas.width = canvasScale * width;
canvas.height = canvasScale * height;

const toggleFieldButton = <HTMLButtonElement>document.getElementById("toggle-field");

toggleFieldButton.addEventListener("click", () => {
  if(postprocField == PostprocField.Temperature){
    postprocField = PostprocField.Phi;
  } else {
    postprocField = PostprocField.Temperature;
  }
  postprocess();
});

const context = canvas.getContext("2d");

const s = new Simulation(width, height);

const postprocess = async () => {
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
}

for(let i = 0; i < 1000; ++i){
  s.step();
  await postprocess();
}


postprocess();
