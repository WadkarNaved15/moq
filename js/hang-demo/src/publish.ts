import "./highlight";

// We need to import Web Components with fully-qualified paths because of tree-shaking.
import HangPublish from "@kixelated/hang/publish/element";
import HangSupport from "@kixelated/hang/support/element";

export { HangPublish, HangSupport };

const publish = document.querySelector("hang-publish") as HangPublish;
const watch = document.getElementById("watch") as HTMLAnchorElement;
const watchName = document.getElementById("watch-name") as HTMLSpanElement;

const urlParams = new URLSearchParams(window.location.search);
const name = urlParams.get("name") ?? "demo/me";
console.log(name);
publish.setAttribute("url", `https://relay.xn--tlay-0ra.com/anon/${name}.hang`);
watch.href = `index.html?name=${name}`;
watchName.textContent = name;