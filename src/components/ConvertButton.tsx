interface ConvertButtonProps {
  fileCount: number;
  isConverting: boolean;
  onConvert: () => void;
}

function ConvertButton({ fileCount, isConverting, onConvert }: ConvertButtonProps) {
  return (
    <button
      className="convert-btn"
      disabled={fileCount === 0 || isConverting}
      onClick={onConvert}
    >
      {isConverting
        ? "Converting..."
        : `Convert ${fileCount} file${fileCount !== 1 ? "s" : ""}`}
    </button>
  );
}

export default ConvertButton;
