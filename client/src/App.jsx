import { useEffect } from "react";
import Memory from "./layout/Memory";
import Navbar from "./layout/Navbar";
import Program from "./layout/Program";
import Registers from "./layout/Registers";
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
