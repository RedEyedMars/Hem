import vsSource  from './res/shaders/vertex.shader';
import fsSource  from './res/shaders/fragment.shader';
import * as glMatrix from 'gl-matrix';
import { onMouseHover, onMouseExit } from './input';


const canvas = document.getElementById("canvas-id");
canvas.style.display = 'inline block';
const gl = canvas.getContext('webgl');
const { mat4, mat3, vec3 } = glMatrix;
var width = canvas.width = window.innerWidth * 0.535;
var height = canvas.height = window.innerHeight-25;
const images = {};
const fetches = {};
let prevFrameTime = Date.now();

const observer = new ResizeObserver(entries => {
  width = canvas.width = window.innerWidth * 0.535;
  height = canvas.height = window.innerHeight-25;
});
observer.observe(document.querySelector('body'));

const shaderProgram = initShaderProgram(gl, vsSource, fsSource);

const glProgramInfo = {
  program: shaderProgram,
  attribLocations: {
    vertexPosition: gl.getAttribLocation(shaderProgram, 'aVertexPosition'),
    textureCoord: gl.getAttribLocation(shaderProgram, 'aTextureCoord'),
  },
  uniformLocations: {
    projectionMatrix: gl.getUniformLocation(shaderProgram, 'uProjectionMatrix'),
    modelViewMatrix: gl.getUniformLocation(shaderProgram, 'uModelViewMatrix'),
    moveMatrix: gl.getUniformLocation(shaderProgram, 'uMoveMatrix'),
    uSampler: gl.getUniformLocation(shaderProgram, 'uSampler'),
  },
};
const glBuffers = initBuffers(gl);

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

let cachegetUint8Memory0 = null
function getUint8Memory() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}
let cachegetInt32Memory0 = null
function getInt32Memory() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}
let cachegetFloat32Memory0 = null
function getFloat32Memory() {
    if (cachegetFloat32Memory0 === null || cachegetFloat32Memory0.buffer !== wasm.memory.buffer) {
        cachegetFloat32Memory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachegetFloat32Memory0;
}

let metaIndex = 0;
let positionStartIndex = 4;
let imageIndex = 2 * 4096 + positionStartIndex;
let game = null;


export function loadImages() {
  const imgs = [
    256 * 1 + 1, // grass
    256 * 2 + 1, // wall
    256 * 16 + 1, // player
    256 * 32 + 1, // monsters
    256 * 128 + 1, // particles
  ];
  imgs.forEach(i => { if(!fetches[i]) fetches[i] = `/i/${i}`;});
  Object.keys(fetches).forEach(i => images[i] = loadTexture(gl, fetches[i]));

  canvas.addEventListener("mousemove", e => {
    e = e || window.event; // IE-ism
    onMouseHover(e.clientX / width, 16 - e.clientY / height );
  });
  canvas.addEventListener("mouseleave", e => {
    onMouseExit();
  });
}
function animate(a_meta, a_img_ids, a_xs, a_ys) {
  drawScene(gl, glProgramInfo, glBuffers, a_meta, a_img_ids, a_xs, a_ys);
}
export default animate;



//
// Initialize a shader program, so WebGL knows how to draw our data
//
function initShaderProgram(gl, vsSource, fsSource) {
  const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource);
  const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource);

  // Create the shader program

  const shaderProgram = gl.createProgram();
  gl.attachShader(shaderProgram, vertexShader);
  gl.attachShader(shaderProgram, fragmentShader);
  gl.linkProgram(shaderProgram);

  // If creating the shader program failed, alert

  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
    alert('Unable to initialize the shader program: ' + gl.getProgramInfoLog(shaderProgram));
    return null;
  }

  return shaderProgram;
}

//
// creates a shader of the given type, uploads the source and
// compiles it.
//
function loadShader(gl, type, source) {
  const shader = gl.createShader(type);

  // Send the source to the shader object

  gl.shaderSource(shader, source);

  // Compile the shader program

  gl.compileShader(shader);

  // See if it compiled successfully

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    alert('An error occurred compiling the shaders: ' + gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}


function initBuffers(gl) {


    // Create a buffer for the cube's vertex positions.

    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      -1.0, -1.0,
       1.0, -1.0,
       1.0,  1.0,
      -1.0,  1.0,
    ]), gl.STATIC_DRAW);

    const [westTextureCoordBuffer, eastTextureCoordBuffer] =
        [[gl.createBuffer(), gl.createBuffer()],
        [gl.createBuffer(), gl.createBuffer()]];
    const [westIndexBuffer, eastIndexBuffer] =
        [gl.createBuffer(), gl.createBuffer()];

    gl.bindBuffer(gl.ARRAY_BUFFER, westTextureCoordBuffer[0]);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      0.0,  1.0,
      0.5,  1.0,
      0.5,  0.0,
      0.0,  0.0,
    ]), gl.STATIC_DRAW);
    gl.bindBuffer(gl.ARRAY_BUFFER, westTextureCoordBuffer[1]);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      0.5,  1.0,
      1.0,  1.0,
      1.0,  0.0,
      0.5,  0.0,
    ]), gl.STATIC_DRAW);

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, westIndexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array([
       0,  1,  2,      0,  2,  3,
    ]), gl.STATIC_DRAW);

    gl.bindBuffer(gl.ARRAY_BUFFER, eastTextureCoordBuffer[0]);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      0.0,  1.0,
      1.0,  1.0,
      1.0,  0.0,
      0.0,  0.0,
    ]), gl.STATIC_DRAW);

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, eastIndexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,
        new Uint16Array([
          3,  1,  2,      1,  3,  0,
        ]), gl.STATIC_DRAW);

    return {
      position: positionBuffer,
      westTextureCoord: westTextureCoordBuffer,
      westIndices: westIndexBuffer,
      eastTextureCoord: eastTextureCoordBuffer,
      eastIndices: eastIndexBuffer,
    };
}

function drawScene(gl, programInfo, buffers, a_meta, a_img_ids, a_xs, a_ys) {

  gl.clearColor(0.0, 0.0, 0.0, 1.0);  // Clear to black, fully opaque
  gl.clearDepth(1.0);                 // Clear everything
  gl.enable(gl.DEPTH_TEST);           // Enable depth testing
  gl.depthFunc(gl.LEQUAL);            // Near things obscure far things
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

  gl.viewport(0, 0, width, height);

  // Clear the canvas before we start drawing on it.

  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  const projectionMatrix = mat4.create();

  // note: glmatrix.js always has the first argument
  // as the destination to receive the result.
  mat4.perspective(projectionMatrix,
                   45 * Math.PI / 180,
                   gl.canvas.clientWidth / gl.canvas.clientHeight,
                   0.1,
                   100.0);

  const modelViewMatrix = mat4.create();

  mat4.translate(modelViewMatrix,     // destination matrix
                 modelViewMatrix,     // matrix to translate
                 [-7.5, -7.5, -38]);  // amount to translate

  {
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.position);
    gl.vertexAttribPointer(
        programInfo.attribLocations.vertexPosition,
        2, // num of components
        gl.FLOAT, // component type
        false, // normalize
        0, // stride
        0); // offset
    gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);
  }

  // Tell WebGL which indices to use to index the vertices
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buffers.westIndices);

  // Tell WebGL to use our program when drawing

  gl.useProgram(programInfo.program);

  // Set the shader uniforms


  // Specify the texture to map onto the faces.
  gl.uniformMatrix4fv(
      programInfo.uniformLocations.projectionMatrix,
      false,
      projectionMatrix);
  gl.uniformMatrix4fv(
      programInfo.uniformLocations.modelViewMatrix,
      false,
      modelViewMatrix);

  // Tell WebGL we want to affect texture unit 0
  gl.activeTexture(gl.TEXTURE0);
  // Tell the shader we bound the texture to texture unit 0
  gl.uniform1i(programInfo.uniformLocations.uSampler, 0);

  const coordPick = Math.floor(prevFrameTime / 400) % 2;
  {
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.westTextureCoord[coordPick]);
    gl.vertexAttribPointer(
        programInfo.attribLocations.textureCoord,
        2, // number of components
        gl.FLOAT, // type
        false, // normalize
        0, // stride
        0); // offset
    gl.enableVertexAttribArray(
        programInfo.attribLocations.textureCoord);
  }

  const numberOfDraws = a_meta[0];
  //console.log("::");
  for(var i = 0; i < numberOfDraws; i += 1) {
    //console.log(a_img_ids[i]);
    drawLayer(programInfo, modelViewMatrix, { image:a_img_ids[i], x: a_xs[i], y: a_ys[i] });
    //drawLayer(programInfo, modelViewMatrix, layer.right, buffers.eastTextureCoord[0]);
  }
}

function drawLayer(programInfo, modelViewMatrix, {image, frame, x, y}) {
  gl.bindTexture(gl.TEXTURE_2D, images[image]);
  const moveMatrix = mat4.create();
  mat4.translate(moveMatrix, modelViewMatrix, [4*x, 4*y, 0.0]);
  gl.uniformMatrix4fv(
      programInfo.uniformLocations.moveMatrix,
      false,
      moveMatrix);
  gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);
}





//
// Initialize a texture and load an image.
// When the image finished loading copy it into the texture.
//
function loadTexture(gl, url) {
  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);

  // Because images have to be download over the internet
  // they might take a moment until they are ready.
  // Until then put a single pixel in the texture so we can
  // use it immediately. When the image has finished downloading
  // we'll update the texture with the contents of the image.
  const level = 0;
  const internalFormat = gl.RGBA;
  const width = 1;
  const height = 1;
  const border = 0;
  const srcFormat = gl.RGBA;
  const srcType = gl.UNSIGNED_BYTE;
  const pixel = new Uint8Array([0, 0, 255, 255]);  // opaque blue
  gl.texImage2D(gl.TEXTURE_2D, level, internalFormat,
                width, height, border, srcFormat, srcType,
                pixel);

  const image = new Image();
  image.onload = function() {
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(gl.TEXTURE_2D, level, internalFormat,
                  srcFormat, srcType, image);

    // WebGL1 has different requirements for power of 2 images
    // vs non power of 2 images so check if the image is a
    // power of 2 in both dimensions.
    if (isPowerOf2(image.width) && isPowerOf2(image.height)) {
       // Yes, it's a power of 2. Generate mips.
       gl.generateMipmap(gl.TEXTURE_2D);
    } else {
       // No, it's not a power of 2. Turn off mips and set
       // wrapping to clamp to edge
       gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
       gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
       gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    }
  };
  image.src = url;

  return texture;
}

function isPowerOf2(value) {
  return (value & (value - 1)) == 0;
}
