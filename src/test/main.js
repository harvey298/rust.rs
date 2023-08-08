const { invoke } = window.__TAURI__.tauri;

var offset_obj;

async function get_servers() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    let result = await invoke("get_servers");
    console.log(result);
}

async function login(profile_id) {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    await invoke("set_profile", {"id": profile_id});
}

async function connect() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    await invoke("connect_to_server", {"id": "69a2f791-636d-4119-9dcf-c1d8495f68b8"});
}

async function connected_test() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    await invoke("get_connected_servers");
}

async function server_map_test() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    await invoke("refresh_map", {"id": "69a2f791-636d-4119-9dcf-c1d8495f68b8" }); // , "offset": Number(offset_obj.value)
}


window.addEventListener("DOMContentLoaded", () => {

    document.querySelector("#server_list_check").addEventListener("click", () => {
        console.log("server_list_check");
        get_servers()
    })

    document.querySelector("#login").addEventListener("click", () => {
        console.log("login");
        let profile_id = document.querySelector("#lname").value;
        login(profile_id)
    })
    
    document.querySelector("#server_connect").addEventListener("click", () => {
        console.log("connect");
        connect()
    })

    document.querySelector("#server_connected_test").addEventListener("click", () => {
        console.log("connected test");
        connected_test()
    })

    document.querySelector("#server_map_test").addEventListener("click", () => {
        console.log("connected map test");
        server_map_test()
    })
    
    offset_obj = document.querySelector("#offset");


});