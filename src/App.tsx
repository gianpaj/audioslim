import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, BaseDirectory } from "@tauri-apps/plugin-fs";

import "./App.css";

function App() {
  const [message, setMessage] = useState("");
  const [loading, setLoading] = useState(false);
  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    e.preventDefault();

    if (!e.currentTarget.files) {
      return;
    }

    const files = Array.from(e.currentTarget.files);

    for (const file of files) {
      if (!file.name.toLowerCase().endsWith(".wav")) {
        setMessage("Please drop a audio file (e.g. `.wav`)");
        return;
      }
      setLoading(true);
      console.log(file);
      setMessage(`Converting ${file.name}...`);

      try {
        // const file = await open("foo/bar.txt", {
        //   read: true,
        //   baseDir: BaseDirectory.App,
        // });
        const result = await invoke("convert_to_mp3", {
          inputPath: `/Users/gianpaj/Downloads/llama-sofia-luna.wav`,
          // inputPath: `${file.webkitRelativePath}/${file.name}`,
        });
        setMessage(`Successfully converted to: ${result}`);
      } catch (error) {
        console.error(error);
        setMessage(`Error: ${error}`);
      }

      setLoading(false);
    }
  };

  // const handleDragOver = (e: React.DragEvent) => {
  //   e.preventDefault();
  // };

  return (
    <div className="container">
      <h2>Drop WAV files here</h2>
      <input
        type="file"
        onChange={handleFileChange}
        disabled={loading}
        accept=".wav, .ogg"
      />
      {/* <div className="dropzone" onDrop={handleDrop} onDragOver={handleDragOver}>
        </div>
        */}
      <p>{message}</p>
      {loading && <p>Converting...</p>}
    </div>
  );
}

export default App;
