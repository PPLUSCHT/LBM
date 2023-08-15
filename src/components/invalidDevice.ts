import { Runner } from "../main"
import { App } from "./app"

export class InvalidDevice extends App{

    constructor(runner: Runner){
        super(runner)

        let errorContainer = document.createElement("div")
        errorContainer.className = "invalid-device-container"

        let errorBox = document.createElement("div")
        errorBox.className = "invalid-device-message"
        errorBox.innerHTML = `Your browser is incompatible with this website. Currently, only the newest versions of edge and chrome for desktop support web gpu (look <a href="https://caniuse.com/webgpu" target="_blank">here</a> to see current browser support). Click <a href="https://github.com/PPLUSCHT/LBM" target="_blank">here</a> to see the source code. `

        errorContainer.appendChild(errorBox)
        document.body.appendChild(errorContainer)
    }

    close(): void {
        this.runner.setInvalid()
    }
}