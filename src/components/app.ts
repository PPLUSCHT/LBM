import { Runner } from "../main";

export abstract class App{
    abstract close(): void;
    runner: Runner;
  
    constructor(runner: Runner){
      this.runner = runner
    }
  }