import init, {Resolution, ClickType, ColorMap, SummaryStat} from 'lbm-wgpu'
import test_compatibility from 'lbm-wgpu'
import { WelcomePage } from './components/welcome';
import { Simulation } from './components/simulation';
import { Tutorial } from './components/tutorial';
import { App } from './components/app';
import "./style.css"

export interface State{
  resolution: Resolution,
  clickType: ClickType,
  color: ColorMap,
  summary: SummaryStat,
  computationSpeed: number,
  viscosity: number,
  stepMode: boolean,
  paused: boolean,
  speed: number
}

export enum Phase{
  Tutorial,
  Simulation, 
  Welcome
}

export interface CurrentLocation{
  phase: Phase, 
  tutorialIndex: number
}

export class Runner{
  
  private currentLocation: CurrentLocation
  private state: State

  constructor(initialState: State, currentLocation: CurrentLocation){
    this.state = initialState
    this.currentLocation = currentLocation
    sessionStorage.setItem("loaded", "true")
    this.createApp(currentLocation)
  }

  stateChange(state: State): void{
    sessionStorage.setItem("state", JSON.stringify(state));
    location.reload()
  }

  exitTutorial(): void{
    let current: CurrentLocation = {
      phase: Phase.Simulation,
      tutorialIndex: 0
    }
    sessionStorage.setItem("currentLocation", JSON.stringify(current))
    location.reload()
  }

  refreshTutorial(): void{
    sessionStorage.setItem("currentLocation", JSON.stringify(this.currentLocation))
    location.reload()
  }

  nextSlideTutorial(): void{
    this.currentLocation.tutorialIndex += 1
    if(this.currentLocation.tutorialIndex >= SLIDE_COUNT){
      this.exitTutorial()
    }else{
      sessionStorage.setItem("currentLocation", JSON.stringify(this.currentLocation))
    }
    location.reload()
  }

  startTutorial(): void{
    let current: CurrentLocation = {
      phase: Phase.Tutorial,
      tutorialIndex: 0
    }
    sessionStorage.setItem("loaded", "true")
    sessionStorage.setItem("currentLocation", JSON.stringify(current))
    location.reload()
  }

  exitWelcome(): void{
    this.exitTutorial()
  }

  returnHome(): void{
    let current: CurrentLocation = {
      phase: Phase.Welcome,
      tutorialIndex: 0
    }
    sessionStorage.setItem("loaded", "true")
    sessionStorage.setItem("currentLocation", JSON.stringify(current))
    location.reload()
  }

  createApp(location: CurrentLocation): App{
    switch (location.phase){
      case Phase.Welcome:
        return new WelcomePage(this)
      case Phase.Simulation:
        sessionStorage.setItem("loaded", "true")
        return new Simulation(this.state, this)
      case Phase.Tutorial:
        return new Tutorial(this, location.tutorialIndex)
    }
  }

}

const SLIDE_COUNT = 7

export const startingState = {
  resolution: Resolution.FHD,
  clickType: ClickType.Draw,
  summary: SummaryStat.Curl,
  computationSpeed: 15,
  viscosity: 0.1,
  color: ColorMap.Jet,
  stepMode: false,
  paused: false,
  speed: 0.1
}

const startingLocation = {
  phase: Phase.Welcome,
  tutorialIndex: 0
}


var _runner: Runner;

function afterLoadCheck(){
  let initialState = sessionStorage.getItem("state") === null ?  startingState : JSON.parse(sessionStorage.getItem("state")!)
  let initialLocation = sessionStorage.getItem("currentLocation") === null ?  startingLocation : JSON.parse(sessionStorage.getItem("currentLocation")!)
  _runner = new Runner(initialState, initialLocation);
}

function resizeRefresh(){
  window.addEventListener("resize", () =>{
    location.reload()
  })
}

window.onload = async () => {
  let loaded:boolean = sessionStorage.getItem("loaded") ?  true : false
  if (loaded == true){
    afterLoadCheck()
    resizeRefresh()
  } else{
    await init()
    let t = await test_compatibility()
    if (!t){
      alert("Your browser is incompatible with this website. Currently, only the newest versions of edge and chrome for desktop work with web gpu")
    }else{
      window.setTimeout(afterLoadCheck, 200)
      window.setTimeout(resizeRefresh, 200)
    }
  }
}
