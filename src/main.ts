import {Resolution, ClickType, ColorMap, SummaryStat} from '../lbm-wgpu/pkg'
import { WelcomePage } from './components/welcome';
import { Simulation } from './components/simulation';
import { Tutorial } from './components/tutorial';
import "./style.css"
import { InvalidDevice } from './components/invalidDevice';

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
  Welcome, 
  invalidDevice
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

  setInvalid(): void{
    let current: CurrentLocation = {
      phase: Phase.invalidDevice,
      tutorialIndex: 0
    }
    sessionStorage.setItem("loaded", "true")
    sessionStorage.setItem("currentLocation", JSON.stringify(current))
    location.reload()
  }

  createApp(location: CurrentLocation){
    switch (location.phase){
      case Phase.Welcome:
        return new WelcomePage(this)
      case Phase.Simulation:
        sessionStorage.setItem("loaded", "true")
        return new Simulation(this.state, this)
      case Phase.Tutorial:
        return new Tutorial(this, location.tutorialIndex)
      case Phase.invalidDevice:
        return new InvalidDevice(this)
    }
  }

  invalidDevice(){

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
  console.log(`loaded: ${loaded}`)
  if (loaded == true){
    afterLoadCheck()
    resizeRefresh()
  } else{
      window.setTimeout(afterLoadCheck, 200)
      window.setTimeout(resizeRefresh, 200)
  }
}

window.onunhandledrejection = function(event){
  console.log(`wasdasfdsf`);
  if(event.reason.toString().includes("Unreachable") || event.reason.toString().includes("unreachable")){
    _runner.setInvalid()
  }
}