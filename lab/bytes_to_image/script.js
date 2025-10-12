// Original: Sample bytes from ICP canister (simulating get_photo response)
const bytes = [
  255, 216, 255, 224, 0, 16, 74, 70, 73, 70, 0, 1, 1, 0, 72, 0, 72, 0, 0, 255, 219, 0, 67, 0, 8, 6, 6, 7, 6, 5, 8, 7, 7,
  7, 9, 9, 8, 10, 12, 14, 13, 12, 11, 11, 12, 25, 18, 19, 15, 20, 29, 26, 31, 30, 29, 26, 28, 28, 32, 36, 46, 39, 32,
  34, 44, 35, 28, 28, 40, 55, 41, 44, 48, 49, 52, 52, 52, 31, 39, 57, 61, 56, 50, 60, 46, 51, 52, 50, 255, 192, 0, 17,
  8, 0, 1, 0, 1, 1, 1, 17, 0, 2, 17, 1, 3, 17, 1, 255, 196, 0, 20, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  8, 255, 196, 0, 20, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 218, 0, 12, 3, 1, 0, 2, 17, 3, 17,
  0, 63, 0, 138, 0, 7, 255, 217,
];

// Display them as plain numbers
document.getElementById("bytes").textContent = bytes;

// You'll see: [255,216,255,224,0,16,74,70,73,70,0,1,1,0,…]
// These are just raw numbers; the browser doesn't "see" an image here.

// Function to display data only when everything is ready
async function renderPage() {
  // Get DOM elements
  const loadingDiv = document.getElementById("loading");
  const bytesDiv = document.getElementById("bytes");
  const arrayBufferDiv = document.getElementById("arrayBuffer");
  const uint8ArrayDiv = document.getElementById("uint8Array");
  const blobDiv = document.getElementById("blob");
  const blobWithTypeFromBytesDiv = document.getElementById("blobWithTypeFromBytes");
  const blobWithTypeFromImageDiv = document.getElementById("blobWithTypeFromImage");
  const blobImg = document.getElementById("blobImg");
  const blobUrlDiv = document.getElementById("blobUrl");
  const blobUrlImg = document.getElementById("blobUrlImg");

  // Display hardcoded bytes immediately
  bytesDiv.textContent = bytes;

  // Load and process the image
  const { arrayBuffer: imageArrayBuffer, uint8Array: imageUint8Array } = await loadImageData();

  // Display the processed data
  arrayBufferDiv.textContent = imageArrayBuffer;
  uint8ArrayDiv.textContent = imageUint8Array;

  // Create and display blob
  const blob = new Blob([new Uint8Array(bytes)]);
  blobDiv.textContent = blob;

  // Create blob from hardcoded bytes
  const blobWithTypeFromBytes = new Blob([new Uint8Array(bytes)], { type: "image/jpeg" });
  blobWithTypeFromBytesDiv.textContent = blobWithTypeFromBytes;

  // Create blob from fetched image data
  const blobWithTypeFromImage = new Blob([imageUint8Array], { type: "image/jpeg" });
  blobWithTypeFromImageDiv.textContent = blobWithTypeFromImage;

  // Try to display blob as img (using fetched data)
  blobImg.src = blobWithTypeFromImage;

  // Create and display blob URL (using fetched data)
  const blobUrl = URL.createObjectURL(blobWithTypeFromImage);
  blobUrlDiv.textContent = blobUrl;

  // Use blob URL in img tag
  blobUrlImg.src = blobUrl;
  console.log("Blob URL:", blobUrl);
  console.log("Blob size from bytes:", blobWithTypeFromBytes.size);
  console.log("Blob size from image:", blobWithTypeFromImage.size);
  console.log("Image bytes length:", imageUint8Array.length);
  console.log("Image arrayBuffer length:", imageArrayBuffer.byteLength);

  // Hide loading and show all content
  loadingDiv.style.display = "none";
  bytesDiv.style.display = "block";
  arrayBufferDiv.style.display = "block";
  uint8ArrayDiv.style.display = "block";
  blobDiv.style.display = "block";
  blobWithTypeFromBytesDiv.style.display = "block";
  blobWithTypeFromImageDiv.style.display = "block";
  blobImg.style.display = "block";
  blobUrlDiv.style.display = "block";
  blobUrlImg.style.display = "block";
}

// Original function: Fetch from assets
async function loadImageFromAssets() {
  const response = await fetch("./assets/avocado_extra_small_22kb.jpg");
  const arrayBuffer = await response.arrayBuffer();
  const bytes = new Uint8Array(arrayBuffer);

  document.getElementById("bytes").textContent = bytes;
  document.getElementById("arrayBuffer").textContent = arrayBuffer;
  document.getElementById("uint8Array").textContent = bytes;
}

// New function: Fetch from assets and convert between formats
async function loadImageData() {
  try {
    const response = await fetch("./assets/avocado_extra_small_22kb.jpg");
    const arrayBuffer = await response.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    return { arrayBuffer, uint8Array };
  } catch (error) {
    console.error("Error loading image:", error);
    // Fallback: create from hardcoded bytes
    const uint8Array = new Uint8Array(bytes);
    const arrayBuffer = uint8Array.buffer;
    return { arrayBuffer, uint8Array };
  }
}

// Start the process when the page loads
document.addEventListener("DOMContentLoaded", renderPage);
