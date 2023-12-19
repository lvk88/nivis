import { Simulation } from 'nivis';
import { Pane } from 'tweakpane';

const canvas = <HTMLCanvasElement>document.getElementById("postproc-area");
const canvasScale = 3;
canvas.width = canvas.clientWidth / canvasScale;
canvas.height = canvas.clientHeight / canvasScale;

canvas.addEventListener('mousedown', (event: MouseEvent) => {
  const rect = canvas.getBoundingClientRect();
  const x = (event.clientX - rect.left) / canvasScale;
  const y = (event.clientY - rect.top) / canvasScale;
  s.add_seed(x, y);
});

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

let animationFrameID : number = null;

const isPaused = () => {
  return animationFrameID == null;
}

const paneContainer = document.getElementById("pane");
const pane = new Pane({container: paneContainer, title: "Controls", expanded: false});
pane.addBinding(simulationParams, 'kappa', {label: "κ", min: 0.8, max: 2.0, step: 0.01} );
pane.addBinding(simulationParams, 'delta', {label: "δ", min: 0.0, max: 0.05, step: 0.005} );
pane.addBinding(simulationParams, 'postprocField', {label: "Show...", options: {phi: "Phi", temperature: "Temperature"} } );
pane.addButton({title: "Clear"}).on('click', () => {
  s.reset();
});
pane.addButton({title: "Random seed"}).on('click', () => {
  randomSeed();
});

const playPauseButton = pane.addButton({title: "Pause"});
playPauseButton.on('click', () => {
  if(!isPaused()){
    cancelAnimationFrame(animationFrameID);
    playPauseButton.title = "Play";
    animationFrameID = null;
  } else {
    playPauseButton.title = "Pause";
    requestAnimationFrame(postprocess);
  }
});

pane.addButton({title: "Export PNG"}).on('click', () => {
  const fakeLink = document.createElement('a');
  fakeLink.download = 'snowflake.png';
  fakeLink.href = canvas.toDataURL();
  fakeLink.click();
});


const postprocess = () => {
  s.kappa = simulationParams.kappa;
  s.delta = simulationParams.delta;
  //console.time("step");
  s.step();
  //console.timeEnd("step");
  let rgbBuffer: Uint8Array = null;
  if(stringToPostprocField.get(simulationParams.postprocField) == PostprocField.Phi){
    rgbBuffer = s.get_phi_rgb();
  } else {
    rgbBuffer = s.get_temperature_rgb();
  }
  const rgbDataArray = new Uint8ClampedArray(rgbBuffer);
  const imageData = new ImageData(rgbDataArray, s.width, s.height);
  createImageBitmap(imageData).then((bitmap) => {
    context.drawImage(bitmap, 0,0, canvas.width, canvas.height);
  });
  animationFrameID = requestAnimationFrame(postprocess);
}

const randomSeed = () => {
  for(let i = 0; i < 10; i++){
    const x = Math.floor(Math.random() * canvas.width - 1);
    const y = Math.floor(Math.random() * canvas.height - 1);
    s.add_seed(x, y);
  }
}

randomSeed();
animationFrameID = requestAnimationFrame(postprocess);
