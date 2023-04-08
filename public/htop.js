import { h, render } from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function App(props) {
    return html`
  <div class="memory">total memory: ${(props.sysinfo.tot_memory / (1024 * 1024 * 1024)).toFixed(2)}GB used: ${(props.sysinfo.used_memory / (1024 * 1024 * 1024)).toFixed(2)} GB</div>
  <div class="memory">total swap: ${props.sysinfo.tot_swap / (1024 * 1024 * 1024)}GB used: ${(props.sysinfo.used_swap / (1024 * 1024 * 1024)).toFixed(2)} GB</div>
  ${props.sysinfo.cpu_util.map(cpu => {
        return html`<div class="bar">
            <div class="bar-inner" style="width: ${cpu}%"></div>
                <label>${cpu.toFixed(0)}%</label>
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