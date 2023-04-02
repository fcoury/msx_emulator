import App from "./js/components/App.mjs";
import Memory from "./js/components/Memory.mjs";
import Navbar from "./js/components/Navbar.mjs";
import Opcode from "./js/components/Opcode.mjs";
import Registers from "./js/components/Registers.mjs";

import { fetchProgram } from "./js/api/index.mjs";

fetchProgram();

lemonade.setComponents({ Navbar, Opcode, Registers, Memory });
lemonade.render(App, document.getElementById("root"));
