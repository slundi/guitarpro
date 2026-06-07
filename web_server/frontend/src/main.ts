import * as alphaTab from "@coderline/alphatab";

const container = document.getElementById("alphatab")!;
const placeholder = document.getElementById("placeholder");

const api = new alphaTab.AlphaTabApi(container, {
  player: {
    enablePlayer: false,
  },
});

api.scoreLoaded.on(() => {
  placeholder?.remove();
});
