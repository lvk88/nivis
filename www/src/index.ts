import { Simulation } from 'mywasm';
import { Pane } from 'tweakpane';

//const width = 150;
//const height = 150;
//const canvasScale = 3;
const canvas = <HTMLCanvasElement>document.getElementById("postproc-area");
const canvasScale = 3;
const width = canvas.clientWidth / canvasScale;
const height = canvas.clientHeight / canvasScale;
const context = canvas.getContext("2d");


const s = new Simulation(width, height);

enum PostprocField{
  Temperature,
  Phi
}

const stringToPostprocField = new Map<string, PostprocField>([
  ["Temperature", PostprocField.Temperature],
  ["Phi", PostprocField.Phi]
]);

const simulationParams = {
  kappa: 1.6,
  delta: 0.04,
  postprocField: "Phi"
};

const paneContainer = document.getElementById("pane-container");

const pane = new Pane({container: paneContainer});
pane.addBinding(simulationParams, 'kappa', {min: 0.8, max: 2.0, step: 0.01} );
pane.addBinding(simulationParams, 'delta', {min: 0.0, max: 0.05, step: 0.005} );
pane.addBinding(simulationParams, 'postprocField', { options: {phi: "Phi", temperature: "Temperature"} } );
pane.addButton({title: "Restart"}).on('click', () => {
  s.reset();
});


const postprocess = async () => {
  s.kappa = simulationParams.kappa;
  s.delta = simulationParams.delta;
  console.time("Simulation");
  s.step();
  console.timeEnd("Simulation");
  let rgbBuffer: Uint8Array = null;
  if(stringToPostprocField.get(simulationParams.postprocField) == PostprocField.Phi){
    rgbBuffer = s.get_phi_rgb();
  } else {
    rgbBuffer = s.get_temperature_rgb();
  }
  const rgbDataArray = new Uint8ClampedArray(rgbBuffer);
  const imageData = new ImageData(rgbDataArray, s.width, s.height);
  const bitmap = await createImageBitmap(imageData);
  const hRatio = canvas.width / bitmap.width;
  const vRatio = canvas.height / bitmap.height;
  const ratio  = Math.min( hRatio, vRatio );
  context.drawImage(bitmap, 0,0, bitmap.width, bitmap.height, 0,0,bitmap.width*ratio, bitmap.height*ratio);
  requestAnimationFrame(postprocess);
}

requestAnimationFrame(postprocess);
