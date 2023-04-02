import { useStore } from "../Store";

export default function Navbar() {
  const fetchStatus = useStore((state) => state.fetchStatus);
  const step = useStore((state) => state.step);
  const status = useStore((state) => state.status);

  const handleRefresh = () => {
    fetchStatus();
  };

  const handleStep = () => {
    step();
  };

  return (
    <div className="navbar">
      <div className="navbar__item">
        <button onClick={handleRefresh}>Refresh</button>
      </div>
      <div className="navbar__item">
        <button onClick={handleStep}>Step</button>
      </div>
    </div>
  );
}
