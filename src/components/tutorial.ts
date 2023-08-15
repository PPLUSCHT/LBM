import { Runner, State, startingState } from "../main"
import init, {run, Resolution, WASMInteraction, FluidPreset, SummaryStat, BarrierPreset} from 'lbm-wgpu'
import { App } from "./app"
import { Simulation } from "./simulation"

abstract class TutorialSlide {
    
    public tutorial: Tutorial

    constructor(tutorial: Tutorial){
        this.tutorial = tutorial
    }

    protected next(): void {
        this.tutorial.nextSlide()
    }

    protected close(): void {
        this.tutorial.close()
    }
}

abstract class SimulationTutorialSlide extends TutorialSlide{

    constructor(tutorial: Tutorial, body: HTMLElement){
        super(tutorial)
        this.initInterface(body)
    }

    protected step(): void {
        WASMInteraction.take_step()
    }

    protected async loop(): Promise<void> {
        while (true){
            this.step()
            await new Promise(r => setTimeout(r, 200))
        }
    }

    protected abstract restart(): void

    private initButtons(): HTMLDivElement{

        let buttonContainer = document.createElement("div")
        buttonContainer.className = "tutorial-button-container"

        buttonContainer.appendChild(this.createButton("Return Home", () => this.close()))
        buttonContainer.appendChild(this.createButton("Restart Simulation", () => this.restart()))
        buttonContainer.appendChild(this.createButton("Next", () => this.next()))
        return buttonContainer
    }

    private initInterface(body: HTMLElement): void{
        
        let userInterface = document.createElement("div")
        userInterface.className = "tutorial-user-interface"
        userInterface.appendChild(body)
        let hr = document.createElement("hr")
        userInterface.appendChild(hr)
        userInterface.appendChild(this.initButtons())
        document.body.appendChild(userInterface)
    }

    private createButton(label: string, clickFn: () => void): HTMLButtonElement{
        let button = document.createElement("button")
        button.onclick = clickFn
        button.innerText = label
        button.id = label.split(" ").join("-").toLowerCase()
        button.className = "tutorial-button"
        return button
    }

}

export class Tutorial extends App{

    slide: TutorialSlide
    index: number

    constructor(runner: Runner, index: number){
        super(runner)
        this.index = index
        console.log("hello")
        this.slide = this.createSlide(index)
    }

    close(): void {
        this.runner.returnHome()
    }

    nextSlide(): void{
        this.runner.nextSlideTutorial()
    }

    createSlide(index: number): TutorialSlide{
        switch (index){
            case 0:
                return new IntroDemo(this, element1())
            case 1:
                return new StreamDemo(this, element2())
            case 2:
                return new CollideDemo(this, element3())
            case 3:
                return new MovingCollideDemo(this, element4())
            case 4:
                return new NoSlipDemo(this, element5())
            case 5:
                return new ChaosDemo(this, element6())
            case 6:
                return new ControllerDemo(this)
            default:
                return new StreamDemo(this, element1())
        }
    }
}

class StreamDemo extends SimulationTutorialSlide{

    constructor(tutorial: Tutorial, body: HTMLElement){
        super(tutorial, body)
        init().then(() => {
            this.restart(); 
            run(window.devicePixelRatio, Resolution.TEST, window.innerWidth, window.innerHeight);
            this.loop()
        })
    }

    restart(): void {
        WASMInteraction.update_viscosity(10000000.0)
        WASMInteraction.change_fluid_preset(FluidPreset.SingleNorth)
        WASMInteraction.set_output(SummaryStat.Rho)
        WASMInteraction.set_step_mode()
    }
}

class CollideDemo extends SimulationTutorialSlide{

    constructor(tutorial: Tutorial, body: HTMLElement){
        super(tutorial, body)
        init().then(() => {
            this.restart(); 
            run(window.devicePixelRatio, Resolution.TEST, window.innerWidth, window.innerHeight);
            this.loop()
        })
    }

    restart(): void {
        WASMInteraction.update_viscosity(0.1)
        WASMInteraction.change_fluid_preset(FluidPreset.SingleOrigin)
        WASMInteraction.set_output(SummaryStat.Rho)
        WASMInteraction.set_step_mode()
    }
}

class MovingCollideDemo extends SimulationTutorialSlide{

    constructor(tutorial: Tutorial, body: HTMLElement){
        super(tutorial, body)
        init().then(() => {
            this.restart(); 
            run(window.devicePixelRatio, Resolution.TEST, window.innerWidth, window.innerHeight);
            this.loop()
        })
    }

    restart(): void {
        WASMInteraction.update_viscosity(0.1)
        WASMInteraction.change_fluid_preset(FluidPreset.SingleEast)
        WASMInteraction.set_output(SummaryStat.Rho)
        WASMInteraction.set_step_mode()
    }
}

class NoSlipDemo extends SimulationTutorialSlide{
    
    constructor(tutorial: Tutorial, body: HTMLElement){
        super(tutorial, body)
        init().then(() => {
            this.restart(); 
            run(window.devicePixelRatio, Resolution.TEST, window.innerWidth, window.innerHeight);
            this.loop()
        })
    }

    restart(): void {
        WASMInteraction.update_viscosity(10000000.0)
        WASMInteraction.change_fluid_preset(FluidPreset.SingleNorthEast)
        WASMInteraction.set_output(SummaryStat.Rho)
        WASMInteraction.set_step_mode()
    }
}

class IntroDemo extends SimulationTutorialSlide{
    
    constructor(tutorial: Tutorial, body: HTMLElement){
        super(tutorial, body)
        init().then(() => {
            this.restart(); 
            run(window.devicePixelRatio, Resolution.HD, window.innerWidth, window.innerHeight);
            this.loop()
        })
    }

    restart(): void {
        WASMInteraction.update_viscosity(0.1)
        WASMInteraction.change_barrier_preset(BarrierPreset.Curl)
        WASMInteraction.change_fluid_preset(FluidPreset.Equilibrium)
        WASMInteraction.set_output(SummaryStat.Speed)
    }
}

class ChaosDemo extends SimulationTutorialSlide{
    
    constructor(tutorial: Tutorial, body: HTMLElement){
        super(tutorial, body)
        init().then(() => {
            this.restart(); 
            run(window.devicePixelRatio, Resolution.FHD, window.innerWidth, window.innerHeight);
            this.loop()
        })
    }

    restart(): void {
        WASMInteraction.update_viscosity(0.1)
        WASMInteraction.change_barrier_preset(BarrierPreset.Chaos)
        WASMInteraction.change_fluid_preset(FluidPreset.Equilibrium)

        WASMInteraction.set_output(SummaryStat.Curl)

    }
}

class ControllerDemo extends TutorialSlide{
    controllerSlide: ControllerSlide
    constructor(tutorial: Tutorial){
        super(tutorial)
        let state = sessionStorage.getItem("controls-demo") != null ? JSON.parse(sessionStorage.getItem("controls-demo")!) : startingState
        this.controllerSlide = new ControllerSlide(this, state)
    }

}

class ControllerSlide extends Simulation{

    private controllerDemo:ControllerDemo

    constructor(controllerDemo: ControllerDemo, state: State){
        super(state, controllerDemo.tutorial.runner)
        this.controllerDemo = controllerDemo
        this.initTutorialPanel()
    }

    override initResolutionChanger(current: Resolution): HTMLSelectElement{
  
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
          sessionStorage.setItem("controls-demo", JSON.stringify(this.state))
          this.controllerDemo.tutorial.runner.refreshTutorial()
        }
    
        return selector
      }

    initTutorialPanel(): HTMLDivElement{
        let container = document.createElement("div")
        container.className = "demo-panel"
        let introParagraph = document.createElement("div")
        introParagraph.innerText = "This is what the actual simulator looks like. Expand one of the panels below if you want to know more about any of the controls."
        introParagraph.className = "demo-panel-intro-paragraph"
        container.appendChild(introParagraph)
        container.appendChild(AccordianFactory().container)
        container.appendChild(document.createElement("hr"))
        container.appendChild(this.initButtons())
        document.body.appendChild(container)
        return container;
    }

    override createButton(label: string, clickFn: () => void): HTMLButtonElement{
        let button = document.createElement("button")
        button.onclick = clickFn
        button.innerText = label
        button.id = label.split(" ").join("-").toLowerCase()
        button.className = "tutorial-button"
        return button
    }
    
    private initButtons(): HTMLDivElement{

        let buttonContainer = document.createElement("div")
        buttonContainer.className = "tutorial-button-container"
        buttonContainer.appendChild(this.createButton("Return Home", () => this.controllerDemo.tutorial.close()))
        buttonContainer.appendChild(this.createButton("Begin Simulating!", () => this.controllerDemo.tutorial.runner.exitTutorial()))
        return buttonContainer
    }

    override handleLoaded(): void {}
}

class AccordianElement{
    
    expanded: boolean
    innerElement: HTMLElement
    container: HTMLDivElement
    index: number
    accordian: AccordianPanel

    constructor(menuText: string, 
                innerElement: HTMLElement, 
                index: number, 
                accordian: AccordianPanel
                ){
        this.expanded = false
        this.innerElement = innerElement
        this.container = this.initInterface(menuText, innerElement)
        this.index = index
        this.accordian = accordian
    }

    expand(): void{
        this.expanded = true
        this.accordian.expand(this.index)
        this.innerElement.style.display = "block"
        this.container.getElementsByClassName("caret")[0].innerHTML = "&#9650;"
    }

    collapse(): void{
        this.expanded = false
        this.accordian.collapse()
        this.innerElement.style.display = "none"
        this.container.getElementsByClassName("caret")[0].innerHTML = "&#9660;"
    }

    clickHandler(): void{
        this.expanded ? this.collapse() : this.expand()
    }

    initInterface(menuText: string, innerElement: HTMLElement): HTMLDivElement{
        let container = document.createElement("div")
        container.className = "accordian-element"
        let button = document.createElement("button")
        button.onclick = () => this.clickHandler()

        let buttonInnerText = document.createElement("span")
        buttonInnerText.className = "accordian-button-text"
        
        let labelSpan = document.createElement("div")
        labelSpan.innerText = menuText
        labelSpan.className = "accordian-button-label"

        let plusSpan = document.createElement("div")
        plusSpan.innerHTML = "&#9660;"
        plusSpan.className = "caret"

        buttonInnerText.appendChild(labelSpan)
        buttonInnerText.appendChild(plusSpan)
        button.appendChild(buttonInnerText)

        button.className = "accordian-button"
        container.appendChild(button)
        container.appendChild(innerElement)
        return container
    }
}

class AccordianPanel{
    elements: AccordianElement[]
    expandedIndex: number | null
    container: HTMLDivElement
    
    constructor(){
        this.elements = []
        this.expandedIndex = null
        this.container = document.createElement("div")
        this.container.className = "accordian-panel"
        document.body.appendChild(this.container)
    }

    collapse(){
        this.expandedIndex = null
    }

    expand(index: number){
        if (this.expandedIndex != null) {
            this.elements[this.expandedIndex].collapse()
        }
        this.expandedIndex = index
    }

    append(a: AccordianElement){
        a.collapse()
        this.elements.push(a)
        this.container.appendChild(a.container)
    }
}

let AccordianFactory = (): AccordianPanel => {
    let accordian = new AccordianPanel()
    let buttonTexts: string[] = ["Color Map", "Draw Type", "Output data", "Lattice cell count", "Computations per frame", "Viscosity", "Speed"]
    let innerElements = accordianInnerElements()
    for (let i = 0; i < buttonTexts.length; i++){
        accordian.append(new AccordianElement(
            buttonTexts[i],
            innerElements[i],
            i,
            accordian
        ))
    }
    return accordian
}

let accordianInnerElements = (): HTMLElement[] => {
    let elements: HTMLElement[] = []

    let e1 = colorMapInnerElement()
    e1.className = "inner-accordian"
    elements.push(e1)

    let e2 = drawTypeInnerElement()
    e2.className = "inner-accordian"
    elements.push(e2)

    let e3 = outputTypeInnerElement()
    e3.className = "inner-accordian"
    elements.push(e3)

    let e4 = laticeCellInnerElement()
    e4.className = "inner-accordian"
    elements.push(e4)

    let e5 = cpfInnerElement()
    e5.className = "inner-accordian"
    elements.push(e5)

    let e6 = viscInnerElement()
    e6.className = "inner-accordian"
    elements.push(e6)

    let e7 = speedInnerElement()
    e7.className = "inner-accordian"
    elements.push(e7)

    return elements
}

let colorMapInnerElement = (): HTMLElement => {
    let text = `The color map selector controls the 
                output of the simulation. The descriptions below 
                describe how each map transistions. For output types 
                with negative values, the far left color represents the
                most negative value, the middle is zero, and the right is the
                most positive. For scalar values, the far left is zero, and the 
                far right is the maximum value.`
    let p = document.createElement('div')
    p.innerText = text


    let colorMapList = document.createElement("ul")

    colorMapList.appendChild(colorRow("Inferno: ", [
        new RGB(253, 255, 165),
        new RGB(189, 56, 85),
        new RGB(0, 0, 5)
    ]))

    colorMapList.appendChild(colorRow("Viridis:", [
        new RGB(254, 232, 38),
        new RGB(34, 145, 141),
        new RGB(68, 2, 85)
    ]))

    colorMapList.appendChild(colorRow("Jet:", [
        new RGB(0, 0, 122),
        new RGB(122, 255, 122),
        new RGB(122, 0, 0)
    ]))

    let container = document.createElement("div")
    container.appendChild(p)
    container.appendChild(colorMapList)

    return container
}

let colorRow = (color: string, colorRGBS: RGB[] ):HTMLLIElement => {
    let li = document.createElement("li")
    
    let span = document.createElement("span")
    span.innerText = color
    li.appendChild(span)

    let colorContainer = document.createElement("div")
    colorContainer.className = 'color-container'

    let s1 = document.createElement("div")
    s1.className = "color-map-square"
    s1.style.backgroundColor = colorRGBS[0].toString()
    colorContainer.appendChild(s1)

    let rarr1 = document.createElement("span")
    rarr1.innerHTML = "&rarr;"
    rarr1.className = "right-arrow"
    colorContainer.appendChild(rarr1)

    let s2 = document.createElement("div")
    s2.className = "color-map-square"
    s2.style.backgroundColor = colorRGBS[1].toString()
    colorContainer.appendChild(s2)

    let rarr2 = document.createElement("span")
    rarr2.innerHTML = "&rarr;"
    rarr2.className = "right-arrow"
    colorContainer.appendChild(rarr2)

    let s3 = document.createElement("div")
    s3.className = "color-map-square"
    s3.style.backgroundColor = colorRGBS[2].toString()
    colorContainer.appendChild(s3)

    li.className = "color-row"

    li.appendChild(colorContainer)
    return li
}

class RGB{
    red: number
    green: number
    blue: number

    constructor(r: number, g: number, b: number){
        this.red = r
        this.green = g
        this.blue = b
    }
    
    toString(): string{
        console.log(`rgb(${this.red}, ${this.blue}, ${this.green})`)
        return `rgb(${this.red},${this.green}, ${this.blue})` 
    }
}

let drawTypeInnerElement = ():HTMLElement => {
    let container = document.createElement('div')
    let text = `The draw type selector allows you to interact with the simulation by creating fluid barriers. The following list descibes each tool.`
    let p = document.createElement('div')
    p.innerText = text

    let ul = document.createElement('ul')

    let li1 = document.createElement('li')
    li1.innerHTML = `Draw: creates contious curves by clicking and dragging.`
    ul.appendChild(li1)

    let li2 = document.createElement('li')
    li2.innerHTML = `Erase: removes barriers by clicking and dragging.`
    ul.appendChild(li2)

    let li3 = document.createElement('li')
    li3.innerHTML = `Line: creates straight lines between two endpoints. Click once to place the first endpoint, and click again to place the second.`
    ul.appendChild(li3)


    container.appendChild(p)
    container.appendChild(ul)
    return container
}

let outputTypeInnerElement = ():HTMLElement => {
    let container = document.createElement('div')
    let text = `The output data selector selects the data to plot. Each cell corresponds to one canvas pixel with a specific color representing the output.`
    let p = document.createElement('div')
    p.innerText = text

    let ul = document.createElement('ul')

    let li1 = document.createElement('li')
    li1.innerHTML = `Curl: a vector quantity roughly defined as the spin around a cell. 
                     Positive curl indicates clockwise spin while negative curl indicates counter-clockwise.`
    ul.appendChild(li1)

    let li2 = document.createElement('li')
    li2.innerHTML = `X Velocity: a vector quantity representing the velocity from left to right.`
    ul.appendChild(li2)

    let li3 = document.createElement('li')
    li3.innerHTML = `Y Velocity: a vector quantity representing the velocity from bottom to top.`
    ul.appendChild(li3)

    let li4 = document.createElement('li')
    li4.innerHTML = `Density: a scalar quantity representing the total of all density vectors in a cell.`
    ul.appendChild(li4)

    let li5 = document.createElement('li')
    li5.innerHTML = `Speed: a scalar quantity representing the fluid speed.`
    ul.appendChild(li5)


    container.appendChild(p)
    container.appendChild(ul)
    return container
}

let laticeCellInnerElement = ():HTMLElement => {
    let text = `The lattice cell selector determines the number of cells in the simulation. The numbers are based on the common screen resolutions nHD, HD, FHD, and UHD.`
    let p = document.createElement('div')
    p.innerText = text
    return p
}

let cpfInnerElement = ():HTMLElement => {
    let text = `This determines the number of streaming and collision steps per render. Increasing this number may make the simulation jittery.`
    let p = document.createElement('div')
    p.innerText = text
    return p
}

let viscInnerElement = ():HTMLElement => {
    let text = `This determines the viscosity of the fluid. In this simulator, increasing the viscosity decreases the dispersion rate in the collision step. This makes the simulation less turbulent and more stable. In the real world, honey is more viscous than water which is more viscous than acetone.`
    let p = document.createElement('div')
    p.innerText = text
    return p
}

let speedInnerElement = ():HTMLElement => {
    let text = `This determines the speed of the fluid. Using this slider resets the fluid to the steady state flow at a given speed. Increasing the speed can lead to chaotic conditions.`
    let p = document.createElement('div')
    p.innerText = text
    return p
}


let element1 = (): HTMLElement => {
    let container = document.createElement("div")
    let p1 = document.createElement("p")
    p1.innerHTML = 'This is a two dimensional "wind" tunnel simulator written using WebAssembly and wgpu. There is a steady flow from the left side of the screen with barriers at the top and bottom. Using the drawing tools, you can make barriers to see how the fluid moves and interacts with different shapes under different fluid conditions.'
    let p2 = document.createElement("p")
    p2.innerHTML = 'Specifically, this is a D2Q9 Lattice Boltzmann method. The simulation takes place on a rectangular grid of square cells. Using a gpu, this simulation can simulate a grid with millions of cells with smooth framerates. For more scientific details about the simulation, refer to the wikipedia page <a href="https://en.wikipedia.org/wiki/Lattice_Boltzmann_methods" target="_blank">here</a>. The following pages in this tutorial will help give a better qualitative understanding of the method.'
    container.appendChild(p1)
    container.appendChild(p2)
    return container
}

let element2 = (): HTMLElement => {
    let container = document.createElement("div")
    let p1 = document.createElement("p")
    p1.innerHTML = "As mentioned in the previous slide, this simulation takes place on a rectangular grid of square cells. Each cell tracks nine “density vectors.” Specifically, each cell tracks a northeast, north, northwest, east, west, southeast, south, southwest, and origin density (D2Q9 means the simulation takes place in two dimensions with nine vectors). The method is split into two steps, a streaming and collision step. To demonstrate the streaming step, there is one cell with a large north density on the right side of the screen. In each computation step, each density vector moves to an adjacent cell. For example, the north density moves to the cell directly above, the east density moves to a cell to the right, etc."
    container.appendChild(p1)
    return container
}

let element3 = (): HTMLElement => {
    let container = document.createElement("div")
    let p1 = document.createElement("p")
    p1.innerHTML = 'The previous slide demonstrated the streaming step. Technically, this step with particle collisions could be used as a physically accurate simulator (see <a href="https://en.wikipedia.org/wiki/Lattice_gas_automaton" target="_blank">Lattice Gas Automaton</a>). However, doing so in any meaningful way would be computationally impossible due to the astronomical number of cells needed. The lattice boltzmann method uses a collision step to make simulations more manageable. Instead of doing individual particle collisions, the lattice boltzmann method moves the current cell density distribution towards an equilibrium distribution. This allows the method to approximate more than one particle per cell.'
    container.appendChild(p1)

    let p2 = document.createElement("p")
    p2.innerHTML = "On the right, simulation starts with one cell with a high origin density in a non-moving fluid. Notice how the density disperses to the neighboring cells in a circular pattern. This is due to the collision step dispersing the origin density to the other density vectors. The directional density vectors then move to the adjacent cells in the streaming step."

    container.appendChild(p2)
    return container
}

let element4 = (): HTMLElement => {
    let container = document.createElement("div")
    let p1 = document.createElement("p")
    p1.innerHTML = "This slide provides an example of a cell with a directional component dispersing. On the right, you can see a simulation of a single cell with a large initial east density. As with the previous example, the density of the initial cell decreases as the density moves to adjacent cells."
    container.appendChild(p1)
    return container
}

let element5 = (): HTMLElement => {
    let container = document.createElement("div")
    let p1 = document.createElement("p")
    p1.innerHTML = "This simulation uses no-slip boundary conditions. Simply put, all density vectors that hit boundaries reflect in the opposite direction. To the right, you can see a simulation with a single cell moving northeast. As described earlier, this cell deflects to the southwest when it hits the top boundary."
    container.appendChild(p1)
    return container
}

let element6 = (): HTMLElement => {
    let container = document.createElement("div")
    let p1 = document.createElement("p")
    p1.innerHTML = `Now that you have a qualitative understanding of the 
                    simulation, let’s talk about its limitations. Firstly, 
                    this simulation uses non-dimensional units, so using it 
                    for serious scientific work is futile. 
                    Although, the code could be modified to use whatever 
                    units you need, and the source code is open-source! 
                    Secondly, this simulation uses 32 bit floating point 
                    numbers. This design decision was made to match the architecture 
                    of most modern GPUs. Unfortunately, low precision floats and inherent
                    algorithmic instability can lead to chaotic results with 
                    high fluid speeds. This can be seen to the right. 
                    If you ever see this result in your own simulation, 
                    reset the fluid to restore normal conditions. 
                    High fluid speeds, low viscosities, and large
                    cell counts increase the probability of this occurring.`
    container.appendChild(p1)
    return container
}
