import {Component, OnInit} from '@angular/core';
import { RouterOutlet } from '@angular/router';

import init, { start } from '../lib/wtd-rust/pkg/wtd_rust.js';

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
