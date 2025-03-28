import "./App.sass";
import "../public/fonts/inter.css"

import { Component } from "solid-js";
import { Sidebar } from "./core/sidebar/Sidebar";
import { Panels } from "./core/panels";

const App: Component = () => {
  return (
    <>
      <Sidebar />
      <Panels />
    </>
  );
};

export default App;
