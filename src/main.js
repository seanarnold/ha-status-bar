const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

function getEntityEmoji(entityId, friendlyName) {
  const s = `${entityId} ${friendlyName}`.toLowerCase();
  if (s.includes("solar") || s.includes(" pv")) return "☀️";
  if (s.includes("battery") || s.includes("powerwall")) return "🔋";
  if (s.includes("grid")) return "⚡";
  if (s.includes("temperature") || s.includes("_temp")) return "🌡️";
  if (s.includes("humidity")) return "💧";
  if (s.includes("rain") || s.includes("precipitation")) return "🌧️";
  if (s.includes("wind")) return "💨";
  if (s.includes("door")) return "🚪";
  if (s.includes("window") || s.includes("blind") || s.includes("cover")) return "🪟";
  if (s.includes("motion") || s.includes("occupancy")) return "🚶";
  if (s.includes("power") || s.includes("watt")) return "⚡";
  if (s.includes("energy") || s.includes("kwh")) return "⚡";
  if (s.includes("lock")) return "🔒";
  if (s.includes("camera")) return "📷";
  if (s.includes("car") || s.includes("_ev") || s.includes("vehicle")) return "🚗";
  if (s.includes("gas")) return "🔥";
  if (s.includes("water")) return "💧";
  if (s.includes("person") || s.includes("presence")) return "👤";
  if (s.includes("smoke") || s.includes("co2") || s.includes("air")) return "🌫️";
  const domain = entityId.split(".")[0];
  const domainMap = {
    light: "💡", switch: "🔌", cover: "🪟", climate: "🌡️",
    water_heater: "🌡️", media_player: "🎵", weather: "⛅",
    alarm_control_panel: "🚨", vacuum: "🤖", fan: "🌀",
    lock: "🔒", camera: "📷", device_tracker: "👤", person: "👤",
    input_boolean: "🔔", binary_sensor: "🔔",
  };
  return domainMap[domain] || "📊";
}

let allEntities = [];
let selectedEntities = [];
let originalSelectedEntities = [];

async function sendPreview() {
  const states = selectedEntities
    .map((id) => allEntities.find((e) => e.entity_id === id))
    .filter(Boolean)
    .map((e) => ({
      entity_id: e.entity_id,
      friendly_name: e.friendly_name,
      state: e.state,
      unit_of_measurement: e.unit_of_measurement,
    }));
  try {
    await invoke("preview_selection", { states });
  } catch (_) {}
}

async function init() {
  const config = await invoke("get_config");
  document.getElementById("ha-url").value = config.ha_url || "";
  document.getElementById("ha-token").value = config.ha_token || "";
  document.getElementById("interval").value = config.refresh_interval_secs || 30;
  selectedEntities = [...(config.selected_entities || [])];
  originalSelectedEntities = [...selectedEntities];

  document.getElementById("btn-test").addEventListener("click", handleTestConnection);
  document.getElementById("btn-save").addEventListener("click", handleSave);
  document.getElementById("entity-filter").addEventListener("input", renderEntityList);

  // Auto-load entity list if credentials are already configured
  if (config.ha_url && config.ha_token) {
    await handleTestConnection();
  }

  // Register after init so originalSelectedEntities is set
  const appWindow = getCurrentWindow();
  await appWindow.onCloseRequested(async (event) => {
    event.preventDefault();
    selectedEntities = [...originalSelectedEntities];
    renderEntityList();
    await sendPreview();
    await appWindow.hide();
  });
}

async function handleTestConnection() {
  const url = document.getElementById("ha-url").value.trim();
  const token = document.getElementById("ha-token").value.trim();
  const statusEl = document.getElementById("test-status");
  const btn = document.getElementById("btn-test");

  btn.disabled = true;
  statusEl.textContent = "Testing...";
  statusEl.className = "";

  try {
    await invoke("test_connection", { url, token });
    statusEl.textContent = "✓ Connected";
    statusEl.className = "status-ok";

    allEntities = await invoke("fetch_entities", { url, token });
    allEntities.sort((a, b) => a.entity_id.localeCompare(b.entity_id));
    renderEntityList();
  } catch (err) {
    statusEl.textContent = "✗ " + err;
    statusEl.className = "status-err";
    allEntities = [];
    renderEntityList();
  } finally {
    btn.disabled = false;
  }
}

function renderEntityList() {
  const filter = document.getElementById("entity-filter").value.toLowerCase();
  const listEl = document.getElementById("entity-list");

  const visible = allEntities.filter(
    (e) =>
      e.entity_id.toLowerCase().includes(filter) ||
      e.friendly_name.toLowerCase().includes(filter)
  );

  if (visible.length === 0) {
    listEl.innerHTML = `<span class="muted">${
      allEntities.length === 0 ? "No entities loaded yet." : "No entities match."
    }</span>`;
    return;
  }

  const grouped = {};
  for (const e of visible) {
    const domain = e.entity_id.split(".")[0];
    if (!grouped[domain]) grouped[domain] = [];
    grouped[domain].push(e);
  }

  let html = "";
  for (const domain of Object.keys(grouped).sort()) {
    html += `<div class="domain-group"><span class="domain-label">${domain}</span>`;
    for (const e of grouped[domain]) {
      const checked = selectedEntities.includes(e.entity_id) ? "checked" : "";
      const unit = e.unit_of_measurement ? ` (${e.unit_of_measurement})` : "";
      const emoji = getEntityEmoji(e.entity_id, e.friendly_name);
      html += `
        <label class="entity-row">
          <input type="checkbox" value="${e.entity_id}" ${checked} />
          <span class="entity-emoji">${emoji}</span>
          <span class="entity-name">${e.friendly_name}</span>
          <span class="entity-meta">${e.entity_id} · ${e.state}${unit}</span>
        </label>`;
    }
    html += "</div>";
  }

  listEl.innerHTML = html;

  listEl.querySelectorAll("input[type=checkbox]").forEach((cb) => {
    cb.addEventListener("change", async () => {
      if (cb.checked) {
        if (!selectedEntities.includes(cb.value)) selectedEntities.push(cb.value);
      } else {
        selectedEntities = selectedEntities.filter((id) => id !== cb.value);
      }
      await sendPreview();
    });
  });
}

async function handleSave() {
  const config = {
    ha_url: document.getElementById("ha-url").value.trim(),
    ha_token: document.getElementById("ha-token").value.trim(),
    selected_entities: selectedEntities,
    refresh_interval_secs: Math.max(
      5,
      parseInt(document.getElementById("interval").value, 10) || 30
    ),
  };

  const btn = document.getElementById("btn-save");
  btn.disabled = true;
  try {
    await invoke("save_config", { config });
    originalSelectedEntities = [...selectedEntities];
    await getCurrentWindow().hide();
  } catch (err) {
    alert("Failed to save: " + err);
    btn.disabled = false;
  }
}

document.addEventListener("DOMContentLoaded", init);
