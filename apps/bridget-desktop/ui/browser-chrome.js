const address = document.getElementById("address");

const navigate = (action, value = "") => {
  const suffix = value ? `?url=${encodeURIComponent(value)}` : "";
  window.location.href = `bridget-browser://${action}${suffix}`;
};

document.getElementById("browser-form").addEventListener("submit", (event) => {
  event.preventDefault();
  let value = address.value.trim();
  if (/^www\./i.test(value)) value = `https://${value}`;
  try {
    const url = new URL(value);
    if (url.protocol !== "https:") throw new Error("HTTPS requis");
    navigate("open", url.href);
  } catch {
    address.setCustomValidity("Saisis une adresse HTTPS valide.");
    address.reportValidity();
    address.setCustomValidity("");
  }
});

for (const button of document.querySelectorAll("[data-action]")) {
  button.addEventListener("click", () => navigate(button.dataset.action));
}

window.__BRIDGET_BROWSER_SET_URL = (url) => {
  address.value = url || "";
};

window.addEventListener("keydown", (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "l") {
    event.preventDefault();
    address.focus();
    address.select();
  }
});
