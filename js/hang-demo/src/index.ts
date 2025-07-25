import "./highlight";

import HangSupport from "@kixelated/hang/support/element";
import HangWatch from "@kixelated/hang/watch/element";

export { HangWatch, HangSupport };

const watch = document.querySelector("hang-watch") as HangWatch;

// If query params are provided, use it as the broadcast name.
const urlParams = new URLSearchParams(window.location.search);
const name = urlParams.get("name") ?? "bbb";
console.log(`https://relay.xn--tlay-0ra.com/anon/${name}.hang`)
watch.setAttribute("url", `https://relay.xn--tlay-0ra.com/anon/${name}.hang`);