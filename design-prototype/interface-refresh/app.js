// Static fixtures only. No IPC, persistence, or domain operations.
const events = [
  {day:0,start:540,end:570,name:"Ana María Rodríguez de los Ángeles",short:"Ana María",service:"Consulta de seguimiento",state:"Confirmada",phone:"+58 412 555 0101"},
  {day:0,start:570,end:630,name:"José Núñez",short:"José Núñez",service:"Evaluación inicial y orientación administrativa",state:"Programada"},
  {day:0,start:660,end:690,name:"Inés Peña",short:"Inés Peña",service:"Consulta de seguimiento",state:"Programada"},
  {day:0,start:720,end:750,name:"Luis Méndez",short:"Luis Méndez",service:"Consulta de seguimiento",state:"Cancelada",kind:"cancelled"},
  {day:1,start:540,end:555,name:"Óscar Díaz",short:"Óscar D.",service:"Consulta breve",state:"Programada"},
  {day:1,start:600,end:637,name:"María Fernanda del Carmen González",short:"María F.",service:"Consulta de seguimiento",state:"Programada"},
  // Parallel fixtures illustrate geometry only; they do not assert a valid reservation.
  {day:2,start:540,end:570,name:"Inés Peña",short:"Inés P.",service:"Consulta de seguimiento",state:"Programada",lane:0,lanes:2},
  {day:2,start:540,end:570,name:"Carlos Méndez",short:"Carlos M.",service:"Consulta de seguimiento",state:"Programada",lane:1,lanes:2},
  {day:3,start:720,end:780,name:"Bloqueo de horario",short:"Bloqueo",service:"No disponible",state:"Bloqueo",kind:"block"},
  {day:4,start:0,end:30,name:"Eva López",short:"Eva López",service:"Consulta de seguimiento",state:"Programada"},
  {day:5,start:1410,end:1440,name:"Raúl Núñez",short:"Raúl Núñez",service:"Consulta de seguimiento",state:"Programada",note:"Inicio de cita: sábado 23:30; fin: domingo 00:30. Duración total: 60 minutos."},
  {day:6,start:0,end:30,name:"Raúl Núñez",short:"Raúl Núñez",service:"Consulta de seguimiento",state:"Programada",note:"Continuación de la cita del sábado 23:30 al domingo 00:30. Duración total: 60 minutos."}
];
const days = ["Lun 9", "Mar 10", "Mié 11", "Jue 12", "Vie 13", "Sáb 14", "Dom 15"];
const grid = document.querySelector("#week-grid");
const detail = document.querySelector("#week-detail");
const scroll = document.querySelector(".calendar-scroll");
const day = document.querySelector("#day");
const week = document.querySelector("#week");
let opener;
let weekVisited = false;
function time(minutes) {
  return `${String(Math.floor(minutes / 60)).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
}
const corner = document.createElement("div");
corner.className = "corner";
corner.textContent = "Hora";
grid.append(corner);
for (const label of days) {
  const header = document.createElement("div");
  header.className = "day-header";
  header.textContent = label;
  grid.append(header);
}
const hours = document.createElement("div");
hours.className = "hours";
for (let hour = 0; hour <= 24; hour++) {
  const label = document.createElement("span");
  label.style.top = `${hour * 96}px`;
  label.textContent = time(hour * 60);
  hours.append(label);
}
grid.append(hours);
for (let index = 0; index < 7; index++) {
  const column = document.createElement("div");
  column.className = "day-column";
  for (const event of events.filter(item => item.day === index)) {
    const button = document.createElement("button");
    button.className = `event ${event.kind || ""}`;
    const label = `${days[index]}, ${time(event.start)}–${time(event.end)}, ${event.name}, ${event.service}, ${event.state}${event.note ? ". " + event.note : ""}`;
    button.setAttribute("aria-label", label);
    button.title = label;
    button.style.top = `${event.start * 1.6}px`;
    button.style.height = `${(event.end - event.start) * 1.6}px`;
    button.style.left = `calc(${(event.lane || 0) * 100 / (event.lanes || 1)}% + 2px)`;
    button.style.width = `calc(${100 / (event.lanes || 1)}% - 4px)`;
    const name = document.createElement("strong");
    name.textContent = event.short;
    const interval = document.createElement("span");
    interval.textContent = `${time(event.start)}–${time(event.end)}`;
    if (event.end - event.start < 30) {
      button.classList.add("brief");
      name.textContent = `${time(event.start)} ${event.short}`;
    }
    button.append(name, interval);
    button.addEventListener("click", () => {
      opener = button;
      detail.replaceChildren();
      const close = document.createElement("button");
      close.textContent = "Volver a Semana";
      close.addEventListener("click", closeDetail);
      const heading = document.createElement("h2");
      heading.textContent = event.name;
      const facts = document.createElement("dl");
      for (const [key, value] of Object.entries({Fecha:days[index],Horario:`${time(event.start)}–${time(event.end)} · Caracas`,Duración:event.note || `${event.end-event.start} minutos`,Servicio:event.service,Estado:event.state,Teléfono:event.phone || "No registrado",Lugar:event.kind === "block" ? "No disponible" : "Consultorio Centro · Avenida Los Leones · Barquisimeto, Lara"})) {
        const term = document.createElement("dt");
        const description = document.createElement("dd");
        term.textContent = key;
        description.textContent = value;
        facts.append(term, description);
      }
      detail.append(close,heading,facts);
      detail.hidden = false;
      detail.focus({preventScroll:true});
    });
    column.append(button);
  }
  grid.append(column);
}
function closeDetail() {
  detail.hidden = true;
  opener?.focus({preventScroll:true});
}
detail.addEventListener("keydown", event => {
  if (event.key === "Escape") closeDetail();
});
const editor = document.querySelector("#editor");
const registration = document.querySelector("#registration");
const booking = document.querySelector("#booking-form");
const patientField = booking.elements.namedItem("patient");
const serviceField = booking.elements.namedItem("service");
const dateField = booking.elements.namedItem("date");
const errorBox = document.querySelector("#booking-errors");
const result = document.querySelector("#booking-result");
const saveButton = document.querySelector("#save-booking");
const slotOptions = document.querySelector("#booking-slots");
const services = {followup:{name:"Consulta de seguimiento",duration:30},initial:{name:"Evaluación inicial y orientación administrativa",duration:60}};
const patientNames = {ana:"Ana María Rodríguez de los Ángeles",ines:"Inés Peña"};
let lastAgenda = "#day";
let returningFromRegistration = false;
let newPatientCount = 0;

const designRoutes = ["#requests", "#waitlist", "#communications", "#metrics", "#team", "#policy"];
const designTitles = {"#requests": "Solicitudes", "#waitlist": "Lista de espera", "#communications": "Comunicaciones", "#metrics": "Indicadores", "#team": "Equipo y dispositivos", "#policy": "Política de comunicaciones"};
function selectView() {
  const route = location.hash;
  const isEditor = route === "#editor";
  const isRegistration = route === "#registration";
  const isWeek = route === "#week";
  const isDesign = designRoutes.includes(route);
  for (const id of designRoutes) document.querySelector(id).hidden = route !== id;
  day.hidden = isWeek || isEditor || isRegistration || isDesign;
  week.hidden = !isWeek;
  editor.hidden = !isEditor;
  registration.hidden = !isRegistration;
  const agendaChromeHidden = isEditor || isRegistration || isDesign;
  document.querySelector("#agenda-heading").hidden = agendaChromeHidden;
  document.querySelector(".views").hidden = agendaChromeHidden;
  document.querySelector("#agenda-toolbar").hidden = agendaChromeHidden;
  document.querySelector("#new-appointment").hidden = agendaChromeHidden;
  document.querySelector("#period-title").textContent = isDesign ? designTitles[route] : isEditor || isRegistration ? "Agenda" : isWeek ? "9–15 de noviembre" : "Lunes, 9 de noviembre";
  for (const link of document.querySelectorAll(".views a")) {
    if (link.hash === (isWeek ? "#week" : "#day")) link.setAttribute("aria-current", "page");
    else link.removeAttribute("aria-current");
  }
  if (!isEditor && !isRegistration) lastAgenda = isWeek ? "#week" : "#day";
  if (isWeek && !weekVisited) {
    scroll.scrollTop = 8 * 96;
    weekVisited = true;
  }
  if (isEditor) {
    document.querySelector(returningFromRegistration ? "#booking-patient" : "#editor-title").focus();
    returningFromRegistration = false;
  }
  if (isRegistration) document.querySelector("#registration-title").focus();
}
function updateSummary() {
  const service = services[serviceField.value];
  const selected = booking.querySelector("input[name=slot]:checked");
  document.querySelector("#summary-patient").textContent = patientNames[patientField.value] || "Sin seleccionar";
  document.querySelector("#summary-service").textContent = service?.name || "Sin seleccionar";
  const dateParts = dateField.value.split("-");
  document.querySelector("#summary-date").textContent = dateField.value ? `${Number(dateParts[2])}/${Number(dateParts[1])}/${dateParts[0]}` : "Sin seleccionar";
  document.querySelector("#summary-slot").textContent = selected && service ? `${time(Number(selected.value))}–${time(Number(selected.value)+service.duration)} · Caracas` : "Sin seleccionar";
  document.querySelector("#summary-duration").textContent = service ? `${service.duration} minutos` : "Sin seleccionar";
}
function renderSlots() {
  slotOptions.replaceChildren();
  const service = services[serviceField.value];
  document.querySelector("#slot-help").textContent = service ? "Selecciona un intervalo." : "Selecciona un servicio para ver los intervalos.";
  if (!service) return;
  // Fixed design options, not a calculation of real availability.
  for (const start of [630,840,930]) {
    const label = document.createElement("label");
    label.className = "slot-choice";
    const radio = document.createElement("input");
    radio.type = "radio";
    radio.name = "slot";
    radio.value = String(start);
    label.append(radio, `${time(start)}–${time(start+service.duration)}`);
    slotOptions.append(label);
  }
}
function clearBookingFeedback() {
  errorBox.hidden = true;
  result.hidden = true;
  saveButton.disabled = false;
  for (const field of [patientField,serviceField,dateField,document.querySelector("#slot-fieldset")]) field.removeAttribute("aria-invalid");
  for (const error of booking.querySelectorAll(".field-error")) error.hidden = true;
}
serviceField.addEventListener("change", renderSlots);
dateField.addEventListener("change", () => {
  // A new date needs an explicit interval choice; other fields remain intact.
  for (const input of slotOptions.querySelectorAll("input")) input.checked = false;
});
booking.addEventListener("input", () => { clearBookingFeedback(); updateSummary(); });
booking.addEventListener("change", () => { clearBookingFeedback(); updateSummary(); });
booking.addEventListener("submit", event => {
  event.preventDefault();
  if (saveButton.disabled) return;
  clearBookingFeedback();
  const selected = booking.querySelector("input[name=slot]:checked");
  const missing = [];
  if (!patientField.value) missing.push([patientField,"patient-error","Selecciona un paciente."]);
  if (!serviceField.value) missing.push([serviceField,"service-error","Selecciona un servicio."]);
  if (!dateField.value) missing.push([dateField,"date-error","Indica una fecha."]);
  if (!selected) missing.push([document.querySelector("#slot-fieldset"),"slot-error","Selecciona un intervalo."]);
  if (missing.length) {
    errorBox.replaceChildren(document.createTextNode("Revisa los campos indicados."));
    for (const [field,id,message] of missing) {
      field.setAttribute("aria-invalid", "true");
      const inline = document.getElementById(id);
      inline.textContent = message;
      inline.hidden = false;
      const link = document.createElement("a");
      link.href = `#${field.id}`;
      link.textContent = message;
      link.addEventListener("click", () => {
        (field === document.querySelector("#slot-fieldset") ? slotOptions.querySelector("input") || serviceField : field).focus();
      });
      errorBox.append(link);
    }
    errorBox.hidden = false;
    (missing[0][0] === document.querySelector("#slot-fieldset") ? slotOptions.querySelector("input") || serviceField : missing[0][0]).focus();
    return;
  }
  if (document.querySelector("#save-scenario").value === "error") {
    errorBox.textContent = "No se pudo guardar la cita. Conservamos los datos. Puedes volver a intentarlo.";
    errorBox.hidden = false;
    return;
  }
  result.textContent = "Cita guardada";
  result.hidden = false;
  saveButton.disabled = true;
});
document.querySelector("#save-scenario").addEventListener("change", clearBookingFeedback);
document.querySelector("#back-agenda").addEventListener("click", () => { location.hash = lastAgenda; });
document.querySelector("#register-patient").addEventListener("click", () => { location.hash = "#registration"; });
function returnToBooking() {
  returningFromRegistration = true;
  location.hash = "#editor";
}
for (const id of ["#cancel-registration","#cancel-registration-bottom"]) document.querySelector(id).addEventListener("click", returnToBooking);
document.querySelector("#registration-form").addEventListener("submit", event => {
  event.preventDefault();
  const form = event.currentTarget;
  const name = form.elements.namedItem("fullName").value.trim();
  if (!name) { form.elements.namedItem("fullName").focus(); return; }
  const key = `example-${++newPatientCount}`;
  patientNames[key] = name;
  const option = document.createElement("option");
  option.value = key;
  option.textContent = `${name} · ${form.elements.namedItem("phoneCode").value} ${form.elements.namedItem("phone").value}`;
  patientField.append(option);
  patientField.value = key;
  clearBookingFeedback();
  updateSummary();
  returnToBooking();
});
window.addEventListener("hashchange", () => {
  if (["#day", "#week", "#editor", "#registration", ...designRoutes].includes(location.hash)) selectView();
});
updateSummary();
selectView();
