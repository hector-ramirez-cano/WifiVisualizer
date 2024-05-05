import { sphere } from "./shape-gen.js"

function initBuffers(gl, sphere) {
    const positionBuffer     = initPositionBuffer(gl, sphere);
    const textureCoordBuffer = initTextureBuffer (gl, sphere);
    const vertexColor        = initColorBuffer   (gl, sphere);
    const indexBuffer        = initIndexBuffer   (gl, sphere);

    return {
        position    : positionBuffer,
        textureCoord: textureCoordBuffer,
        color       : vertexColor,
        indices     : indexBuffer,
    };
}

function initPositionBuffer(gl, sphere) {
    // Create a buffer for the square's positions.
    const positionBuffer = gl.createBuffer();

    // Select the positionBuffer as the one to apply buffer
    // operations to from here out.
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);

    // Now pass the list of positions into WebGL to build the
    // shape. We do this by creating a Float32Array from the
    // JavaScript array, then use it to fill the current buffer.
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(sphere.positions), gl.STATIC_DRAW);

    return positionBuffer;
}

function initColorBuffer(gl, sphere) {
    const faceColors = [
        [1.0, 1.0, 1.0, 1.0], // Front face: white
        [1.0, 0.0, 0.0, 1.0], // Back face: red
        [0.0, 1.0, 0.0, 1.0], // Top face: green
        [0.0, 0.0, 1.0, 1.0], // Bottom face: blue
        [1.0, 1.0, 0.0, 1.0], // Right face: yellow
        [1.0, 0.0, 1.0, 1.0], // Left face: purple
    ];

    // Convert the array of colors into a table for all the vertices.

    var colors = [];

    const colorBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, colorBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(sphere.colors), gl.STATIC_DRAW);

    return colorBuffer;
}

function initIndexBuffer(gl, sphere) {
    const indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);

    // This array defines each face as two triangles, using the
    // indices into the vertex array to specify each triangle's
    // position.

    
    // Now send the element array to GL

    gl.bufferData(
        gl.ELEMENT_ARRAY_BUFFER,
        new Uint16Array(sphere.triangles),
        gl.STATIC_DRAW
    );

    return indexBuffer;
}

function initTextureBuffer(gl, sphere) {
    const textureCoordBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, textureCoordBuffer);


    gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array(sphere.uvs),
        gl.STATIC_DRAW
    );

    return textureCoordBuffer;
}

export { initBuffers };