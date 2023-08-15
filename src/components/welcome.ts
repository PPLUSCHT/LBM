import { Runner } from "../main"
import init, {run, Resolution, WASMInteraction, SummaryStat, ColorMap, BarrierPreset} from '../../lbm-wgpu/pkg'
import { App } from "./app"

export class WelcomePage extends App{

    constructor(runner: Runner){
        super(runner)
        init().then(() => {
            this.updateWASMState()
            this.initWelcomeButtons()
            run(window.devicePixelRatio, Resolution.HD, window.innerWidth, window.innerHeight)
        })
    }

    private async fadeWelcome(){
        WASMInteraction.change_barrier_preset(BarrierPreset.Tunnel)
        await new Promise(r => setTimeout(r, 1500))
    }

    async close(): Promise<void> {
        await this.fadeWelcome()
        this.runner.exitWelcome()
    }

    private async startTutorial(): Promise<void>{
        await this.fadeWelcome()
        this.runner.startTutorial()
    }

    private updateWASMState(){
        WASMInteraction.set_color_map(ColorMap.Viridis)
        WASMInteraction.set_output(SummaryStat.Speed)
        WASMInteraction.change_barrier_preset(BarrierPreset.Welcome)
    }

    private initWelcomeButtons(): void{
        let buttonContainer = document.createElement("div")
        buttonContainer.className = "welcome-button-container"
        buttonContainer.appendChild(this.createButton("Tutorial", () => this.startTutorial()))
        buttonContainer.appendChild(this.createButton("Simulator", () => this.close()))
        document.body.appendChild(buttonContainer)
    }

    private createButton(label: string, clickFn: () => void): HTMLButtonElement{
        let button = document.createElement("button")
        button.onclick = clickFn
        button.innerText = label
        button.id = label.split(" ").join("-").toLowerCase()
        button.className = "welcome-button"
        return button
    }
}