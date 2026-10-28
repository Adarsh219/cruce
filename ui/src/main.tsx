import React from "react"
import ReactDOM from "react-dom/client"
import { CruceApp } from "./components/app-shell"
import "./index.css"

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <CruceApp />
  </React.StrictMode>,
)
