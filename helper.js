// Set the textarea's text to the value of the localStorage config
// TODO get default value if LS item is not set
let configTextArea;
let configSaveBtn;
window.onload = () => {
  // Get elements
  configTextArea = document.getElementById("config");
  configSaveBtn = document.getElementById("configSave");

  // Register save button's onClick event
  configSaveBtn.onclick = saveConfigToLocalStorage;

  let currConfig = window.localStorage.getItem("config");
  configTextArea.value = currConfig;
};

function saveConfigToLocalStorage() {
  // Get current textbox value and save to LS
  let val = configTextArea.value;
  window.localStorage.setItem("config", val);
  console.debug("Saved config to local storage.");
  location.reload();
}
