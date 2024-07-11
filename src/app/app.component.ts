import {Component, OnInit} from '@angular/core';
import { RouterOutlet } from '@angular/router';

import init, { start } from './pkg/walk_the_dog_rust.js';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent implements OnInit {
  title = 'walk-the-dog';
  ngOnInit() {
    init().then(() => {
      console.log("WASM module loaded");
      start();
    });
  }
}
