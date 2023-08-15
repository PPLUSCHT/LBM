import init, {run, Resolution, WASMInteraction, ClickType, ColorMap, SummaryStat, FluidPreset, BarrierPreset} from '../../lbm-wgpu/pkg'
import { Runner, State } from '../main'
import { App } from './app'

import closeURL from '../public/close.svg'

export class Simulation extends App{

    private expanded: boolean
    protected state: State
    private container: HTMLDivElement
    private panel: HTMLDivElement
    private expandButton: HTMLElement
  
    constructor(state: State, runner: Runner){
      super(runner)

      init().then(() => {
        this.updateWASMState(state); 
        run(window.devicePixelRatio, state.resolution, window.innerWidth, window.innerHeight);
      })
      this.state = state
      this.expanded = false
      this.container = this.initContainer()
      this.panel = this.initPanel(state)
      this.expandButton = this.initCollapsedButtons()
      this.expand()
      this.handleLoaded()
    }

    protected handleLoaded(): void{
      let loaded:boolean = sessionStorage.getItem("simulationLoaded") ?  true : false
      if (loaded != true){
        alert("Resizing the browser window or changing the lattice cell count resets the simulation! Only change these if you are comfortable losing your progress.")
        sessionStorage.setItem("simulationLoaded", "true")
      }
    }
  
    private updateWASMState(state: State){
      WASMInteraction.set_color_map(state.color)
      WASMInteraction.set_draw_type(state.clickType)
      WASMInteraction.update_viscosity(state.viscosity)
      WASMInteraction.set_output(state.summary)
      WASMInteraction.update_compute_rate(state.computationSpeed)
      WASMInteraction.update_flow_speed(state.speed)
      if (state.stepMode) {
        WASMInteraction.set_step_mode()
      }else{
        WASMInteraction.release_step_mode()
      }

      if (state.paused){
        WASMInteraction.toggle_pause()
      }
    }
  
    private initContainer():  HTMLDivElement{
      let container = document.body.appendChild(document.createElement("div"))
      container.className = "container"
      return container
    }

    private initCollapsedButtons(): HTMLDivElement{
      let container = document.createElement("div")
      container.className = "settings-buttons"
      container.appendChild(this.createSVGButton("public/vite.svg","settings", () => this.expansionButtonHandler()))
      container.appendChild(this.createSVGButton("public/home.svg","settings", () => this.runner.returnHome()))
      return container;
    }
  
    private initPanel(state: State): HTMLDivElement{
      let panel = document.createElement("div")
      panel.className = "panel"
      panel.appendChild(this.createSVGButton(closeURL, "collapse", () => this.expansionButtonHandler()))
      panel.appendChild(this.createLabeledRow("Color Map", "dropdown-row", this.initColorMapSelector(state.color)))
      panel.appendChild(this.createLabeledRow("Draw Type", "dropdown-row", this.initClickTypeSelector(state.clickType)))
      panel.appendChild(this.createLabeledRow("Output Data", "dropdown-row", this.initOutputSelector(state.summary)))
      panel.appendChild(this.createLabeledRow("Lattice Cell Count", "dropdown-row",this.initResolutionChanger(state.resolution)))
      panel.appendChild(document.createElement("hr"))
      panel.appendChild(this.createSliderRow("Computations Per Frame", "slider-row", this.initComputeSlider(state.computationSpeed)))
      panel.appendChild(this.createSliderRow("Viscosity", "slider-row",this.initViscositySlider(state.viscosity)))
      panel.appendChild(this.createSliderRow("Speed", "slider-row",this.initSpeedSlider(state.speed)))
      panel.appendChild(document.createElement("hr"))
      panel.appendChild(this.setupButtons())
      return panel
    }
  
    private initColorMapSelector(current: ColorMap): HTMLSelectElement{
      let selector = document.createElement("select")
  
      let enumVals = Object.keys(ColorMap).filter(key => isNaN(Number(key)))
  
      let options = enumVals.map((v, i) => {
        let option = document.createElement("option")
        option.value = i.toString()
        option.innerHTML = v
        return option
      })
  
      options.forEach( o => selector.appendChild(o))
  
      selector.selectedIndex = current
  
      selector.onchange = () => this.setColor(parseInt(selector.value))
  
      return selector
    }
  
    private setColor(c: ColorMap){
      this.state.color = c
      WASMInteraction.set_color_map(c)
    }
  
    protected initResolutionChanger(current: Resolution): HTMLSelectElement{
  
      function option(text: string, value: number): HTMLOptionElement{
        let o = document.createElement("option")
        o.value = value.toString()
        o.innerHTML = text
        return o
      }
      let selector = document.createElement("select")
  
      selector.appendChild(option(Resolution.NHD.toString(), Resolution.NHD));
      selector.appendChild(option(Resolution.HD.toString(), Resolution.HD));
      selector.appendChild(option(Resolution.FHD.toString(), Resolution.FHD));
      selector.appendChild(option(Resolution.UHD.toString(), Resolution.UHD));
  
      switch(current){
        case (Resolution.NHD):
          selector.selectedIndex = 0
          break;
        case (Resolution.HD):
          selector.selectedIndex = 1
          break;
        case (Resolution.FHD):
          selector.selectedIndex = 2
          break;
        case (Resolution.UHD):
          selector.selectedIndex = 3
          break;
        default:
          selector.selectedIndex = 0
      }
  
      selector.onchange = () => {
        this.state.resolution = parseInt(selector.value)
        this.close()
      }
  
      return selector
    }
  
    public close(): void{
      this.runner.stateChange(this.state)
    }
  
    private initComputeSlider(current: number): HTMLInputElement{
      let input = document.createElement("input")
      input.type = "range"
      input.min = "1"
      input.max = "50"
      input.value = current.toString()
      input.step = "1"
      input.onchange = () => this.setComputeSpeed(parseInt(input.value))
      return input
    }
  
    private setComputeSpeed(rate: number){
      this.state.computationSpeed = rate
      WASMInteraction.update_compute_rate(rate)
    }
  
    private initViscositySlider(current: number): HTMLInputElement{
      let input = document.createElement("input")
      input.type = "range"
      input.min = "0.005"
      input.max = "0.3"
      input.step = "0.005"
      input.value = current.toString()
      input.onchange = () => this.setViscosity(parseFloat(input.value))
      return input
    }

    private initSpeedSlider(current: number): HTMLInputElement{
      let input = document.createElement("input")
      input.type = "range"
      input.min = "0.0"
      input.max = "0.3"
      input.step = "0.005"
      input.value = current.toString()
      input.onchange = () => this.setSpeed(parseFloat(input.value))
      return input
    }
  
    private setViscosity(v: number){
      this.state.viscosity = v
      WASMInteraction.update_viscosity(v)
    }

    private setSpeed(v: number){
      this.state.speed = v
      WASMInteraction.update_flow_speed(v)
    }
  
  
    private initOutputSelector(current: SummaryStat): HTMLSelectElement{
  
      function option(text: string, value: number): HTMLOptionElement{
        let o = document.createElement("option")
        o.value = value.toString()
        o.innerHTML = text
        return o
      }
      let selector = document.createElement("select")
  
      selector.appendChild(option("Curl", SummaryStat.Curl))
      selector.appendChild(option("X Velocity", SummaryStat.Ux))
      selector.appendChild(option("Y Velocity", SummaryStat.Uy))
      selector.appendChild(option("Density", SummaryStat.Rho))
      selector.appendChild(option("Speed", SummaryStat.Speed))
  
      selector.selectedIndex = current;
      selector.onchange = () => this.setSummaryStat(parseInt(selector.value))
      return selector
    }
  
    private setSummaryStat(summ: SummaryStat){
      this.state.summary = summ
      WASMInteraction.set_output(summ)
    }
  
    private createLabeledRow(label: string, className: string, element: HTMLElement): HTMLDivElement{
      let container = document.createElement("div")
      container.className = className
      let spanElement = document.createElement("span")
      spanElement.innerText = label
      container.appendChild(spanElement)
      container.appendChild(element)
      return container;
    }
  
    private createSliderRow(label: string, className: string, element: HTMLInputElement): HTMLDivElement{
      let container = document.createElement("div")
      container.className = className
      let spanElement = document.createElement("span")
      spanElement.innerText = label
      spanElement.className = "slider-label"
      container.appendChild(spanElement)
      container.appendChild(element)
      let output = document.createElement("span")
      output.className = "output"
      output.innerText = element.value
      element.className = "slider"
      element.oninput = () => {output.innerText = element.value}
      container.appendChild(output)
      return container;
    }
  
    private initClickTypeSelector(current: ClickType): HTMLSelectElement{
      let selector = document.createElement("select")
      let enumVals = Object.keys(ClickType).filter(key => isNaN(Number(key)))
  
      let options = enumVals.map((v, i) => {
        let option = document.createElement("option")
        option.value = i.toString()
        option.innerHTML = v
        return option
      })
  
      options.forEach( o => selector.appendChild(o))
  
      selector.selectedIndex = current
      selector.onchange = () => this.setClickType(parseInt(selector.value))
      return selector
    }
  
    private setClickType(type: ClickType){
      this.state.clickType = type
      WASMInteraction.set_draw_type(type)
    }
  
    protected createButton(label: string, clickFn: () => void): HTMLButtonElement{
      let button = document.createElement("button")
      button.onclick = clickFn
      button.innerText = label
      button.id = label.split(" ").join("-").toLowerCase()
      button.className = "panel-button"
      return button
    }
  
    private createSVGButton(path: string, id: string, clickFn: () => void): HTMLButtonElement{
      let button = document.createElement("button")
      button.onclick = clickFn
      button.className = "svg-button"
      button.id = id
      let svg = document.createElement("img")
      svg.src = path
      svg.alt = id
      svg.id = id
      button.appendChild(svg)
      return button
    }
  
    private setupButtons(): HTMLDivElement{
      
      let buttonContainer = document.createElement("div")
      buttonContainer.className = "button-container"
      buttonContainer.appendChild(this.createButton("Undo", () => WASMInteraction.undo()))
      buttonContainer.appendChild(this.createButton("Pause", () => {
        WASMInteraction.toggle_pause()
        this.state.paused = !this.state.paused
        if(this.state.paused){
          document.getElementById("pause")!.style.color = "aqua"
        } else{
          document.getElementById("pause")!.style.color = "white"
        }
      }))
      buttonContainer.appendChild(this.createButton("Reset Fluid", () => WASMInteraction.change_fluid_preset(FluidPreset.Equilibrium)))
      buttonContainer.appendChild(this.createButton("Reset Barrier", () => WASMInteraction.change_barrier_preset(BarrierPreset.Tunnel)))
  
      return buttonContainer
    }
  
    private expansionButtonHandler(): void{
      this.expanded ? this.collapse() : this.expand()
    }
    
    private expand() : void{
      this.expanded = true
      this.container.replaceChildren(this.panel)
    }
    
    private collapse() : void{
      this.expanded = false
      this.container.replaceChildren(this.expandButton)
    }
  
  }