const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

async function listen_for_pair_notifcations() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  await invoke("listen_for_pair_notifcations");
}

async function steam_login() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  await invoke("steam_login");
}

window.addEventListener("DOMContentLoaded", () => {
  var steam_login_button = document.querySelector("#steam-login");
  steam_login_button.addEventListener("click", () => {
    console.log("steam login");
    steam_login()

  });

  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");

  var listen_button = document.querySelector("#listen-button");

  listen_button.addEventListener("click", () => {listen_button.disabled = true;listen_for_pair_notifcations() }  );


});

