import { h, render } from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function App(props) {
    return html`
  <div class="memory">memory ${props.sysinfo.tot_memory / (1024 * 1024 * 1024)} GB</div>
  <div class="memory">swap ${props.sysinfo.tot_swap / (1024 * 1024 * 1024)} GB</div>
  ${props.sysinfo.cpu_util.map(cpu => {
        return html`<div class="bar">
            <div class="bar-inner" style="width: ${cpu}%"></div>
            <label>${cpu.toFixed(2)}%</label>
            </div>`;
    })}
  });  
  `;
}

let i = 0;

let update = async () => {
    let response = await fetch("/api/v1/sysinfo");
    if (response.status !== 200) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }

    let json = await response.json();
    render(html`<${App} sysinfo=${json}></${App}>`, document.body);
};

update();
setInterval(update, 1500);