import { CPU } from "viper";

// define some constants 
const frameScale = 0.5;         // frame image scaling factor
const frameRelativeSize = 0.8;  // frame size relative to window


// define the canvas
const canvas = document.getElementById("display-canvas");
const context = canvas.getContext('2d');

// fetch the image
const frameImg = new Image();
frameImg.src = "img/frame.png";

frameImg.onload = function (e)
{
    // get frame image size 
    const frameWidth = frameScale * frameImg.width;
    const frameHeight = frameScale * frameImg.height;

    // set the internal width and height of canvas
    canvas.width = frameWidth;
    canvas.height = frameHeight;

    // scale the canvas element to take up the correct amount of the screen
    const scale = frameRelativeSize * Math.min(
        window.innerHeight / frameHeight, 
        window.innerWidth / frameWidth
    );
    canvas.style.width = `${frameWidth * scale}px`;
    canvas.style.height = `${frameHeight * scale}px`;

    // finally, draw the frame
    context.drawImage(frameImg, 0, 0, frameWidth, frameHeight);
}

const cpu = CPU.new();

const romSelector = document.getElementById('rom-selector');

romSelector.addEventListener('change', (event) => {
    const reader = new FileReader();
    const romFile = event.target.files[0];

    reader.onload = () => {
        const romData = new Uint8Array(reader.result);
        cpu.load(romData);
        // requestAnimationFrame(renderLoop);
    }

    reader.readAsArrayBuffer(romFile);
});

/*
const renderLoop = () => {
    emulatorDisplay.textContent = cpu.render()
    cpu.step();

    requestAnimationFrame(renderLoop);
}

*/