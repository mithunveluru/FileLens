import "./spinner.css";

interface SpinnerProps {
  label?: string;
}

function Spinner({ label = "Loading" }: SpinnerProps) {
  return <span className="spinner" role="status" aria-label={label} />;
}

export default Spinner;
