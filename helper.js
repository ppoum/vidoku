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

  registerAccordions();
};

function saveConfigToLocalStorage() {
  // Get current textbox value and save to LS
  let val = configTextArea.value;
  window.localStorage.setItem("config", val);
  console.debug("Saved config to local storage.");
  location.reload();
}

function registerAccordions() {
  var accs = document.getElementsByClassName("accordion");
  for (let i = 0; i < accs.length; i++) {
    accs[i].addEventListener("click", () => {
      accs[i].classList.toggle("active");

      // Toggle between block and unblocked display
      let panel = accs[i].nextElementSibling;
      if (panel.style.display === "block") {
        panel.style.display = "none";
      } else {
        panel.style.display = "block";
      }
    });
  }
}
