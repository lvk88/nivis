import { Simulation } from 'mywasm';

const width = 100;
const height = 100;

const canvasScale = 3;

const canvas = <HTMLCanvasElement>document.getElementById("postproc-area");
canvas.width = canvasScale * width;
canvas.height = canvasScale * height;

const context = canvas.getContext("2d");

const s = new Simulation(width, height);

const rgbBuffer = s.get_phi_rgb();
const rgbDataArray = new Uint8ClampedArray(rgbBuffer);
const imageData = new ImageData(rgbDataArray, width, height);
const bitmap = await createImageBitmap(imageData);
context.drawImage(bitmap, 0, 0, 3 * width, 3 * height);
