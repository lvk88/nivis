import { Simulation } from 'mywasm';
import { Pane } from 'tweakpane';

const canvas = <HTMLCanvasElement>document.getElementById("postproc-area");
const canvasScale = 3;
canvas.width = canvas.clientWidth / canvasScale;
canvas.height = canvas.clientHeight / canvasScale;

console.log(canvas.clientWidth+ " x " + canvas.clientHeight);

const context = canvas.getContext("2d");


const s = new Simulation(canvas.width, canvas.height);

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
pane.addBinding(simulationParams, 'kappa', {label: "κ", min: 0.8, max: 2.0, step: 0.01} );
pane.addBinding(simulationParams, 'delta', {label: "δ", min: 0.0, max: 0.05, step: 0.005} );
pane.addBinding(simulationParams, 'postprocField', {label: "Show...", options: {phi: "Phi", temperature: "Temperature"} } );
pane.addButton({title: "Restart"}).on('click', () => {
  s.reset();
});


const postprocess = async () => {
  s.kappa = simulationParams.kappa;
  s.delta = simulationParams.delta;
  s.step();
  let rgbBuffer: Uint8Array = null;
  if(stringToPostprocField.get(simulationParams.postprocField) == PostprocField.Phi){
    rgbBuffer = s.get_phi_rgb();
  } else {
    rgbBuffer = s.get_temperature_rgb();
  }
  const rgbDataArray = new Uint8ClampedArray(rgbBuffer);
  const imageData = new ImageData(rgbDataArray, s.width, s.height);
  const bitmap = await createImageBitmap(imageData);
  //const ratio  = Math.min( hRatio, vRatio );
  context.drawImage(bitmap, 0,0, canvas.width, canvas.height);
  requestAnimationFrame(postprocess);
}

requestAnimationFrame(postprocess);

window.onresize = ()=>{
  console.log(window.innerWidth + " + " + window.innerHeight);
};
