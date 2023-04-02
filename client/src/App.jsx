import { useEffect } from "react";
import Memory from "./components/Memory";
import Navbar from "./components/Navbar";
import Program from "./components/Program";
import Registers from "./components/Registers";
import { useStore } from "./Store";

function App() {
  const fetchStatus = useStore((state) => state.fetchStatus);

  useEffect(() => {
    fetchStatus();
  }, []);

  return (
    <div className="container">
      <Navbar />
      <div className="main">
        <Program />
        <div className="status">
          <Registers />
          <div className="split">
            <Memory />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
