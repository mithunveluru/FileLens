import "./spinner.css";

interface SpinnerProps {
  /** Accessible label announced to screen readers. */
  label?: string;
}

/** A small indeterminate loading spinner. */
function Spinner({ label = "Loading" }: SpinnerProps) {
  return <span className="spinner" role="status" aria-label={label} />;
}

export default Spinner;
