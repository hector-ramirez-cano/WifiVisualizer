import { initBuffers } from "./init-buffers.js";
import { drawScene   } from "./draw-scene.js";
import { initShaderProgram  } from "./shader.js";
import { loadTexture } from "./texture.js";
import { sphere } from "./shape-gen.js";

let cubeRotation = 0.0;
let deltaTime = 0;

function resizeCanvasToDisplaySize(canvas) {
    // Lookup the size the browser is displaying the canvas in CSS pixels.
    const displayWidth  = canvas.clientWidth;
    const displayHeight = canvas.clientHeight;

    // Check if the canvas is not the same size.
    const needResize = (canvas.width+1)  !== displayWidth ||
                        canvas.height !== displayHeight;

    if (needResize) {
      // Make the canvas the same size
        canvas.width  = displayWidth;
        canvas.height = displayHeight;
    }

    return needResize;
}

main();

//
// start here
//
function main() {
    const canvas = document.querySelector("#glcanvas");
    // Initialize the GL context
    const gl = canvas.getContext("webgl");

    // Only continue if WebGL is available and working
    if (gl === null) {
        alert(
        "Unable to initialize WebGL. Your browser or machine may not support it."
        );
        return;
    }
    

    // Vertex shader program
    const vsSource = `
        attribute vec4 aVertexPosition;
        attribute vec4 aVertexColor;
        attribute vec2 aTextureCoord;

        uniform mat4 uModelViewMatrix;
        uniform mat4 uProjectionMatrix;

        varying highp vec2 vTextureCoord;
        varying highp vec4 vColor;

        void main(void) {
            gl_Position   = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
            vTextureCoord = aTextureCoord;
            vColor        = aVertexColor;
        }
    `;

    // Fragment shader program
    const fsSource = `
        varying highp vec2 vTextureCoord;
        varying highp vec4 vColor;

        uniform sampler2D uSampler;

        void main(void) {
            gl_FragColor = vec4(texture2D(uSampler, vTextureCoord).xyz * vColor.xyz, 1.0);
        }
    `;

    // Initialize a shader program; this is where all the lighting
    // for the vertices and so forth is established.
    const shaderProgram = initShaderProgram(gl, vsSource, fsSource);

    // Collect all the info needed to use the shader program.
    // Look up which attributes our shader program is using
    // for aVertexPosition, aVertexColor and also
    // look up uniform locations.
    const programInfo = {
            program        : shaderProgram,
            attribLocations: {
            vertexPosition : gl.getAttribLocation(shaderProgram, "aVertexPosition"),
            textureCoord   : gl.getAttribLocation(shaderProgram, "aTextureCoord"  ),
            vertexColor    : gl.getAttribLocation(shaderProgram, "aVertexColor"   )
        },
            uniformLocations: {
            projectionMatrix: gl.getUniformLocation(
                shaderProgram,
                "uProjectionMatrix"
        ),
            modelViewMatrix: gl.getUniformLocation(shaderProgram, "uModelViewMatrix"),
            uSampler: gl.getUniformLocation(shaderProgram, "uSampler"),
        },
    };

    const s = sphere(70);

    // Here's where we call the routine that builds all the
    // objects we'll be drawing.
    const buffers = initBuffers(gl, s);

    // Load texture
    const texture = loadTexture(gl, "res/raster/earth.jpg");
    console.log(texture);
    // Flip image pixels into the bottom-to-top order that WebGL expects.
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);

    let then = 0;

    // Draw the scene repeatedly
    function render(now) {
        now *= 0.001; // convert to seconds     

        resizeCanvasToDisplaySize(gl.canvas);
        gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

        deltaTime = now - then;
        then = now;

        drawScene(gl, programInfo, buffers, texture, s, cubeRotation);
        cubeRotation += deltaTime;

        requestAnimationFrame(render);
    }
    requestAnimationFrame(render);
}
