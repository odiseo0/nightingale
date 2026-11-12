'use strict';

// A separate visual study: all records and outcomes below are fictional.
const TODAY = '2026-10-22';
// Phosphor 2.1.10, regular: local SVG paths; MIT license in assets/icons-LICENSE.txt.
const icons = {
  calendar: "<path d=\"M208,32H184V24a8,8,0,0,0-16,0v8H88V24a8,8,0,0,0-16,0v8H48A16,16,0,0,0,32,48V208a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V48A16,16,0,0,0,208,32ZM72,48v8a8,8,0,0,0,16,0V48h80v8a8,8,0,0,0,16,0V48h24V80H48V48ZM208,208H48V96H208V208Z\"></path>",
  people: "<path d=\"M117.25,157.92a60,60,0,1,0-66.5,0A95.83,95.83,0,0,0,3.53,195.63a8,8,0,1,0,13.4,8.74,80,80,0,0,1,134.14,0,8,8,0,0,0,13.4-8.74A95.83,95.83,0,0,0,117.25,157.92ZM40,108a44,44,0,1,1,44,44A44.05,44.05,0,0,1,40,108Zm210.14,98.7a8,8,0,0,1-11.07-2.33A79.83,79.83,0,0,0,172,168a8,8,0,0,1,0-16,44,44,0,1,0-16.34-84.87,8,8,0,1,1-5.94-14.85,60,60,0,0,1,55.53,105.64,95.83,95.83,0,0,1,47.22,37.71A8,8,0,0,1,250.14,206.7Z\"></path>",
  inbox: "<path d=\"M208,32H48A16,16,0,0,0,32,48V208a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V48A16,16,0,0,0,208,32Zm0,16V152h-28.7A15.86,15.86,0,0,0,168,156.69L148.69,176H107.31L88,156.69A15.86,15.86,0,0,0,76.69,152H48V48Zm0,160H48V168H76.69L96,187.31A15.86,15.86,0,0,0,107.31,192h41.38A15.86,15.86,0,0,0,160,187.31L179.31,168H208v40Z\"></path>",
  clock: "<path d=\"M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm64-88a8,8,0,0,1-8,8H128a8,8,0,0,1-8-8V72a8,8,0,0,1,16,0v48h48A8,8,0,0,1,192,128Z\"></path>",
  message: "<path d=\"M128,24A104,104,0,0,0,36.18,176.88L24.83,210.93a16,16,0,0,0,20.24,20.24l34.05-11.35A104,104,0,1,0,128,24Zm0,192a87.87,87.87,0,0,1-44.06-11.81,8,8,0,0,0-6.54-.67L40,216,52.47,178.6a8,8,0,0,0-.66-6.54A88,88,0,1,1,128,216Z\"></path>",
  chart: "<path d=\"M224,200h-8V40a8,8,0,0,0-8-8H152a8,8,0,0,0-8,8V80H96a8,8,0,0,0-8,8v40H48a8,8,0,0,0-8,8v64H32a8,8,0,0,0,0,16H224a8,8,0,0,0,0-16ZM160,48h40V200H160ZM104,96h40V200H104ZM56,144H88v56H56Z\"></path>",
  settings: "<path d=\"M128,80a48,48,0,1,0,48,48A48.05,48.05,0,0,0,128,80Zm0,80a32,32,0,1,1,32-32A32,32,0,0,1,128,160Zm109.94-52.79a8,8,0,0,0-3.89-5.4l-29.83-17-.12-33.62a8,8,0,0,0-2.83-6.08,111.91,111.91,0,0,0-36.72-20.67,8,8,0,0,0-6.46.59L128,41.85,97.88,25a8,8,0,0,0-6.47-.6A112.1,112.1,0,0,0,54.73,45.15a8,8,0,0,0-2.83,6.07l-.15,33.65-29.83,17a8,8,0,0,0-3.89,5.4,106.47,106.47,0,0,0,0,41.56,8,8,0,0,0,3.89,5.4l29.83,17,.12,33.62a8,8,0,0,0,2.83,6.08,111.91,111.91,0,0,0,36.72,20.67,8,8,0,0,0,6.46-.59L128,214.15,158.12,231a7.91,7.91,0,0,0,3.9,1,8.09,8.09,0,0,0,2.57-.42,112.1,112.1,0,0,0,36.68-20.73,8,8,0,0,0,2.83-6.07l.15-33.65,29.83-17a8,8,0,0,0,3.89-5.4A106.47,106.47,0,0,0,237.94,107.21Zm-15,34.91-28.57,16.25a8,8,0,0,0-3,3c-.58,1-1.19,2.06-1.81,3.06a7.94,7.94,0,0,0-1.22,4.21l-.15,32.25a95.89,95.89,0,0,1-25.37,14.3L134,199.13a8,8,0,0,0-3.91-1h-.19c-1.21,0-2.43,0-3.64,0a8.08,8.08,0,0,0-4.1,1l-28.84,16.1A96,96,0,0,1,67.88,201l-.11-32.2a8,8,0,0,0-1.22-4.22c-.62-1-1.23-2-1.8-3.06a8.09,8.09,0,0,0-3-3.06l-28.6-16.29a90.49,90.49,0,0,1,0-28.26L61.67,97.63a8,8,0,0,0,3-3c.58-1,1.19-2.06,1.81-3.06a7.94,7.94,0,0,0,1.22-4.21l.15-32.25a95.89,95.89,0,0,1,25.37-14.3L122,56.87a8,8,0,0,0,4.1,1c1.21,0,2.43,0,3.64,0a8.08,8.08,0,0,0,4.1-1l28.84-16.1A96,96,0,0,1,188.12,55l.11,32.2a8,8,0,0,0,1.22,4.22c.62,1,1.23,2,1.8,3.06a8.09,8.09,0,0,0,3,3.06l28.6,16.29A90.49,90.49,0,0,1,222.9,142.12Z\"></path>",
  search: "<path d=\"M229.66,218.34l-50.07-50.06a88.11,88.11,0,1,0-11.31,11.31l50.06,50.07a8,8,0,0,0,11.32-11.32ZM40,112a72,72,0,1,1,72,72A72.08,72.08,0,0,1,40,112Z\"></path>",
  plus: "<path d=\"M224,128a8,8,0,0,1-8,8H136v80a8,8,0,0,1-16,0V136H40a8,8,0,0,1,0-16h80V40a8,8,0,0,1,16,0v80h80A8,8,0,0,1,224,128Z\"></path>",
  left: "<path d=\"M165.66,202.34a8,8,0,0,1-11.32,11.32l-80-80a8,8,0,0,1,0-11.32l80-80a8,8,0,0,1,11.32,11.32L91.31,128Z\"></path>",
  right: "<path d=\"M181.66,133.66l-80,80a8,8,0,0,1-11.32-11.32L164.69,128,90.34,53.66a8,8,0,0,1,11.32-11.32l80,80A8,8,0,0,1,181.66,133.66Z\"></path>",
  down: "<path d=\"M213.66,101.66l-80,80a8,8,0,0,1-11.32,0l-80-80A8,8,0,0,1,53.66,90.34L128,164.69l74.34-74.35a8,8,0,0,1,11.32,11.32Z\"></path>",
  more: "<path d=\"M140,128a12,12,0,1,1-12-12A12,12,0,0,1,140,128Zm56-12a12,12,0,1,0,12,12A12,12,0,0,0,196,116ZM60,116a12,12,0,1,0,12,12A12,12,0,0,0,60,116Z\"></path>",
  close: "<path d=\"M205.66,194.34a8,8,0,0,1-11.32,11.32L128,139.31,61.66,205.66a8,8,0,0,1-11.32-11.32L116.69,128,50.34,61.66A8,8,0,0,1,61.66,50.34L128,116.69l66.34-66.35a8,8,0,0,1,11.32,11.32L139.31,128Z\"></path>",
  check: "<path d=\"M229.66,77.66l-128,128a8,8,0,0,1-11.32,0l-56-56a8,8,0,0,1,11.32-11.32L96,188.69,218.34,66.34a8,8,0,0,1,11.32,11.32Z\"></path>",
  help: "<path d=\"M140,180a12,12,0,1,1-12-12A12,12,0,0,1,140,180ZM128,72c-22.06,0-40,16.15-40,36v4a8,8,0,0,0,16,0v-4c0-11,10.77-20,24-20s24,9,24,20-10.77,20-24,20a8,8,0,0,0-8,8v8a8,8,0,0,0,16,0v-.72c18.24-3.35,32-17.9,32-35.28C168,88.15,150.06,72,128,72Zm104,56A104,104,0,1,1,128,24,104.11,104.11,0,0,1,232,128Zm-16,0a88,88,0,1,0-88,88A88.1,88.1,0,0,0,216,128Z\"></path>",
  arrow: "<path d=\"M221.66,133.66l-72,72a8,8,0,0,1-11.32-11.32L196.69,136H40a8,8,0,0,1,0-16H196.69L138.34,61.66a8,8,0,0,1,11.32-11.32l72,72A8,8,0,0,1,221.66,133.66Z\"></path>",
  pin: "<path d=\"M128,64a40,40,0,1,0,40,40A40,40,0,0,0,128,64Zm0,64a24,24,0,1,1,24-24A24,24,0,0,1,128,128Zm0-112a88.1,88.1,0,0,0-88,88c0,31.4,14.51,64.68,42,96.25a254.19,254.19,0,0,0,41.45,38.3,8,8,0,0,0,9.18,0A254.19,254.19,0,0,0,174,200.25c27.45-31.57,42-64.85,42-96.25A88.1,88.1,0,0,0,128,16Zm0,206c-16.53-13-72-60.75-72-118a72,72,0,0,1,144,0C200,161.23,144.53,209,128,222Z\"></path>",
  phone: "<path d=\"M222.37,158.46l-47.11-21.11-.13-.06a16,16,0,0,0-15.17,1.4,8.12,8.12,0,0,0-.75.56L134.87,160c-15.42-7.49-31.34-23.29-38.83-38.51l20.78-24.71c.2-.25.39-.5.57-.77a16,16,0,0,0,1.32-15.06l0-.12L97.54,33.64a16,16,0,0,0-16.62-9.52A56.26,56.26,0,0,0,32,80c0,79.4,64.6,144,144,144a56.26,56.26,0,0,0,55.88-48.92A16,16,0,0,0,222.37,158.46ZM176,208A128.14,128.14,0,0,1,48,80,40.2,40.2,0,0,1,82.87,40a.61.61,0,0,0,0,.12l21,47L83.2,111.86a6.13,6.13,0,0,0-.57.77,16,16,0,0,0-1,15.7c9.06,18.53,27.73,37.06,46.46,46.11a16,16,0,0,0,15.75-1.14,8.44,8.44,0,0,0,.74-.56L168.89,152l47,21.05h0s.08,0,.11,0A40.21,40.21,0,0,1,176,208Z\"></path>",
  link: "<path d=\"M240,88.23a54.43,54.43,0,0,1-16,37L189.25,160a54.27,54.27,0,0,1-38.63,16h-.05A54.63,54.63,0,0,1,96,119.84a8,8,0,0,1,16,.45A38.62,38.62,0,0,0,150.58,160h0a38.39,38.39,0,0,0,27.31-11.31l34.75-34.75a38.63,38.63,0,0,0-54.63-54.63l-11,11A8,8,0,0,1,135.7,59l11-11A54.65,54.65,0,0,1,224,48,54.86,54.86,0,0,1,240,88.23ZM109,185.66l-11,11A38.41,38.41,0,0,1,70.6,208h0a38.63,38.63,0,0,1-27.29-65.94L78,107.31A38.63,38.63,0,0,1,144,135.71a8,8,0,0,0,16,.45A54.86,54.86,0,0,0,144,96a54.65,54.65,0,0,0-77.27,0L32,130.75A54.62,54.62,0,0,0,70.56,224h0a54.28,54.28,0,0,0,38.64-16l11-11A8,8,0,0,0,109,185.66Z\"></path>",
  device: "<path d=\"M208,40H48A24,24,0,0,0,24,64V176a24,24,0,0,0,24,24h72v16H96a8,8,0,0,0,0,16h64a8,8,0,0,0,0-16H136V200h72a24,24,0,0,0,24-24V64A24,24,0,0,0,208,40ZM48,56H208a8,8,0,0,1,8,8v80H40V64A8,8,0,0,1,48,56ZM208,184H48a8,8,0,0,1-8-8V160H216v16A8,8,0,0,1,208,184Z\"></path>",
  shield: "<path d=\"M208,40H48A16,16,0,0,0,32,56v56c0,52.72,25.52,84.67,46.93,102.19,23.06,18.86,46,25.26,47,25.53a8,8,0,0,0,4.2,0c1-.27,23.91-6.67,47-25.53C198.48,196.67,224,164.72,224,112V56A16,16,0,0,0,208,40Zm0,72c0,37.07-13.66,67.16-40.6,89.42A129.3,129.3,0,0,1,128,223.62a128.25,128.25,0,0,1-38.92-21.81C61.82,179.51,48,149.3,48,112l0-56,160,0ZM82.34,141.66a8,8,0,0,1,11.32-11.32L112,148.69l50.34-50.35a8,8,0,0,1,11.32,11.32l-56,56a8,8,0,0,1-11.32,0Z\"></path>",
  filter: "<path d=\"M200,136a8,8,0,0,1-8,8H64a8,8,0,0,1,0-16H192A8,8,0,0,1,200,136Zm32-56H24a8,8,0,0,0,0,16H232a8,8,0,0,0,0-16Zm-80,96H104a8,8,0,0,0,0,16h48a8,8,0,0,0,0-16Z\"></path>",
  menu: "<path d=\"M224,128a8,8,0,0,1-8,8H40a8,8,0,0,1,0-16H216A8,8,0,0,1,224,128ZM40,72H216a8,8,0,0,0,0-16H40a8,8,0,0,0,0,16ZM216,184H40a8,8,0,0,0,0,16H216a8,8,0,0,0,0-16Z\"></path>",
  edit: "<path d=\"M227.31,73.37,182.63,28.68a16,16,0,0,0-22.63,0L36.69,152A15.86,15.86,0,0,0,32,163.31V208a16,16,0,0,0,16,16H92.69A15.86,15.86,0,0,0,104,219.31L227.31,96a16,16,0,0,0,0-22.63ZM92.69,208H48V163.31l88-88L180.69,120ZM192,108.68,147.31,64l24-24L216,84.68Z\"></path>",
  wifi: "<path d=\"M140,204a12,12,0,1,1-12-12A12,12,0,0,1,140,204ZM237.08,87A172,172,0,0,0,18.92,87,8,8,0,0,0,29.08,99.37a156,156,0,0,1,197.84,0A8,8,0,0,0,237.08,87ZM205,122.77a124,124,0,0,0-153.94,0A8,8,0,0,0,61,135.31a108,108,0,0,1,134.06,0,8,8,0,0,0,11.24-1.3A8,8,0,0,0,205,122.77Zm-32.26,35.76a76.05,76.05,0,0,0-89.42,0,8,8,0,0,0,9.42,12.94,60,60,0,0,1,70.58,0,8,8,0,1,0,9.42-12.94Z\"></path>",
  info: "<path d=\"M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm16-40a8,8,0,0,1-8,8,16,16,0,0,1-16-16V128a8,8,0,0,1,0-16,16,16,0,0,1,16,16v40A8,8,0,0,1,144,176ZM112,84a12,12,0,1,1,12,12A12,12,0,0,1,112,84Z\"></path>",
  grid: "<path d=\"M104,40H56A16,16,0,0,0,40,56v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V56A16,16,0,0,0,104,40Zm0,64H56V56h48v48Zm96-64H152a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V56A16,16,0,0,0,200,40Zm0,64H152V56h48v48Zm-96,32H56a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V152A16,16,0,0,0,104,136Zm0,64H56V152h48v48Zm96-64H152a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V152A16,16,0,0,0,200,136Zm0,64H152V152h48v48Z\"></path>"
};
const icon = name => `<svg class="i" viewBox="0 0 256 256" aria-hidden="true" focusable="false">${icons[name] || icons.calendar}</svg>`;
const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const initials = name => name.trim().split(/\s+/).slice(0,2).map(s=>s[0]).join('');
const avatarTones=['sky','violet','rose','teal','orange','cyan'];
function avatar(name,personId=name){
  let hash=0;
  for(const char of personId)hash=(hash*31+char.charCodeAt(0))>>>0;
  // Stable by identity: filtering, editing a name and reopening a view keep the same tone.
  return `<span class="avatar avatar--${avatarTones[hash%avatarTones.length]}" data-avatar-id="${esc(personId)}" aria-hidden="true">${esc(initials(name))}</span>`;
}
const href = route => `#/${route}`;
const link = (text, route, style='') => `<a class="${style}" href="${href(route)}">${text}</a>`;
const button = (text, action, style='', extra='') => `<button type="button" class="${style}" data-action="${action}" ${extra}>${text}</button>`;
const statusClass = status => ({Programada:'scheduled',Programado:'scheduled',Confirmada:'success',Entregado:'success',Ausencia:'attention',Cancelada:'cancelled','Solicitud de cambio':'attention','Revisar teléfono':'attention'})[status]||'';
const badge = status => `<span class="badge ${statusClass(status)}">${esc(status)}</span>`;
const fmt = value => new Intl.DateTimeFormat('es-VE',{day:'numeric',month:'long',timeZone:'UTC'}).format(new Date(value+'T12:00:00Z'));
const longDate = value => new Intl.DateTimeFormat('es-VE',{day:'numeric',month:'long',year:'numeric',timeZone:'UTC'}).format(new Date(value+'T12:00:00Z'));
const fullDate = value => new Intl.DateTimeFormat('es-VE',{weekday:'long',day:'numeric',month:'long',timeZone:'UTC'}).format(new Date(value+'T12:00:00Z'));
const titleCase = text => text.charAt(0).toUpperCase()+text.slice(1);
const patientSeed = [
  {id:'p1',name:'Ana María Rodríguez',phoneCode:'+58',phoneNumber:'412 000 1001',email:'ana@example.test',contact:'WhatsApp',visits:4,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000100',status:'declared'}},
  {id:'p2',name:'Luis Fernando Pérez',phoneCode:'+58',phoneNumber:'414 000 1002',email:'luis@example.test',contact:'WhatsApp',visits:2,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000101',status:'declared'}},
  {id:'p3',name:'María Fernanda del Carmen González',phoneCode:'+58',phoneNumber:'424 000 1003',email:'maria@example.test',contact:'WhatsApp',visits:6,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000102',status:'declared'}},
  {id:'p4',name:'Carlos Méndez',phoneCode:'+58',phoneNumber:'416 000 1004',email:'carlos@example.test',contact:'WhatsApp',visits:1,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000103',status:'declared'}},
  {id:'p5',name:'Valentina Herrera',phoneCode:'+58',phoneNumber:'412 000 1005',email:'valentina@example.test',contact:'WhatsApp',visits:3,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000104',status:'declared'}},
  {id:'p6',name:'José Antonio Castillo',phoneCode:'+58',phoneNumber:'414 000 1006',email:'jose@example.test',contact:'WhatsApp',visits:2,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000105',status:'declared'}},
  {id:'p7',name:'Sofía Rivas',phoneCode:'+58',phoneNumber:'424 000 1007',email:'sofia@example.test',contact:'WhatsApp',visits:1,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000106',status:'declared'}},
  {id:'p8',name:'Andrés Gutiérrez',phoneCode:'+58',phoneNumber:'416 000 1008',email:'andres@example.test',contact:'WhatsApp',visits:5,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'0000107',status:'declared'}},
  {id:'p9',name:'Gabriel Rivas',phoneCode:'',phoneNumber:'',email:'',contact:'Responsable',visits:1,birthDate:'2015-05-18',needsResponsible:true,responsibleIds:['c1','c2'],completion:'complete',identificationBasis:'minor_without_document'},
  {id:'p10',name:'Ricardo Salas',phoneCode:'',phoneNumber:'',email:'',contact:'No definida',visits:0,completion:'draft'},
  {id:'p11',name:'Ana María Rodríguez',phoneCode:'+58',phoneNumber:'414 555 0182',email:'',contact:'WhatsApp',visits:0,completion:'complete',identificationBasis:'own_document',identifier:{country:'VE',type:'national_id',number:'V-00984127',status:'declared'}},
  {id:'p12',name:'Beatriz Luna',phoneCode:'+58',phoneNumber:'',email:'',contact:'No definida',visits:0,completion:'draft',identificationBasis:'own_document',identifier:{country:'VE',type:'passport',number:'A-00223344',status:'declared'}}
];
const appointmentSeed = [
  {id:'a1',patient:'p1',date:TODAY,start:'09:00',end:'09:30',service:'Consulta general',status:'Confirmada'},
  {id:'a2',patient:'p2',date:TODAY,start:'09:30',end:'10:00',service:'Control de seguimiento',status:'Programada'},
  {id:'a3',patient:'p3',date:TODAY,start:'10:00',end:'10:30',service:'Consulta general',status:'Programada'},
  {id:'a4',patient:'p4',date:TODAY,start:'11:00',end:'11:30',service:'Primera consulta',status:'Confirmada'},
  {id:'a5',patient:'p5',date:TODAY,start:'11:30',end:'12:00',service:'Control de seguimiento',status:'Confirmada'},
  {id:'a6',patient:'p6',date:TODAY,start:'14:00',end:'14:30',service:'Consulta general',status:'Programada'},
  {id:'a7',patient:'p7',date:TODAY,start:'14:30',end:'15:00',service:'Primera consulta',status:'Confirmada'},
  {id:'a8',patient:'p8',date:TODAY,start:'15:00',end:'15:30',service:'Control de seguimiento',status:'Confirmada'},
  {id:'a9',patient:'p1',date:'2026-10-23',start:'09:00',end:'09:30',service:'Consulta general',status:'Programada'},
  {id:'a10',patient:'p6',date:'2026-10-21',start:'10:00',end:'10:30',service:'Consulta general',status:'Atendida'},
  {id:'a11',patient:'p4',date:'2026-10-20',start:'11:00',end:'11:30',service:'Primera consulta',status:'Cancelada'},
  {id:'a12',patient:'p3',date:'2026-10-19',start:'14:00',end:'14:30',service:'Control de seguimiento',status:'Atendida'},
  {id:'a13',patient:'p2',date:'2026-10-19',start:'09:00',end:'10:00',service:'Primera consulta',status:'Atendida'},
  {id:'a14',patient:'p5',date:'2026-10-20',start:'10:00',end:'10:45',service:'Consulta general',status:'Atendida'},
  {id:'a15',patient:'p8',date:'2026-10-21',start:'11:00',end:'11:15',service:'Control de seguimiento',status:'Atendida'},
  {id:'a16',patient:'p7',date:'2026-10-23',start:'14:00',end:'14:45',service:'Primera consulta',status:'Programada'},
  {id:'a17',patient:'p8',date:'2026-10-23',start:'14:45',end:'15:45',service:'Consulta general',status:'Confirmada'},
  {id:'a18',patient:'p4',date:'2026-10-21',start:'11:00',end:'11:45',service:'Consulta general',status:'Atendida',professionalId:'martin'},
  {id:'a19',patient:'p2',date:'2026-10-21',start:'15:30',end:'16:00',service:'Consulta general',status:'Ausencia'},
  {id:'a20',patient:'p1',date:'2026-10-05',start:'09:00',end:'09:30',service:'Consulta general',status:'Atendida'},
  {id:'a21',patient:'p1',date:'2026-10-07',start:'09:00',end:'09:30',service:'Consulta general',status:'Atendida'},
  {id:'a22',patient:'p1',date:'2026-10-09',start:'09:30',end:'10:00',service:'Control de seguimiento',status:'Atendida'},
  {id:'a23',patient:'p1',date:'2026-10-12',start:'09:00',end:'09:30',service:'Consulta general',status:'Atendida'},
  {id:'a24',patient:'p1',date:'2026-10-14',start:'09:00',end:'09:30',service:'Consulta general',status:'Atendida'},
  {id:'a25',patient:'p1',date:'2026-10-16',start:'11:00',end:'11:30',service:'Consulta general',status:'Atendida'},
  {id:'a26',patient:'p1',date:'2026-10-17',start:'09:00',end:'09:30',service:'Primera consulta',status:'Atendida'}
];
const serviceCatalog=[
  {id:'s1',name:'Consulta general',baseDuration:30,active:true},
  {id:'s2',name:'Control de seguimiento',baseDuration:30,active:true},
  {id:'s3',name:'Primera consulta',baseDuration:45,active:true}
];
// La oferta vincula el catálogo del profesional con esta consulta: ajuste de duración y visibilidad.
const serviceOffers=[
  {id:'o_s1',serviceId:'s1',override:null,active:true},
  {id:'o_s2',serviceId:'s2',override:45,active:true},
  {id:'o_s3',serviceId:'s3',override:null,active:false}
];
const offerByServiceId = serviceId => serviceOffers.find(o=>o.serviceId===serviceId);
const offeredNames = () => serviceOffers.filter(o=>o.active).map(o=>serviceCatalog.find(s=>s.id===o.serviceId)).filter(s=>s&&s.active!==false).map(s=>s.name);
const teamMembers=[
  {id:'staff-elena',name:'Elena Arias',email:'elena@example.test',role:'Médica',access:'Dueña · gestiona la consulta, el equipo y la configuración',active:true},
  {id:'staff-lucia',name:'Lucía Torres',email:'lucia@example.test',role:'Asistente',access:'Agenda, pacientes y comunicaciones; sin cambios de configuración',active:true}
];
const invitations=[
  {id:'inv1',email:'pedro@example.test',role:'Asistente',sent:'2026-10-20',status:'pending'},
  {id:'inv2',email:'maria@example.test',role:'Asistente',sent:'2026-10-12',status:'expired'}
];
const devices=[
  {id:'dev-elena',name:'Equipo de Elena',os:'Windows',app:'Nightingale 0.1',last:'Ahora',authorized:'2026-09-14 · Elena Arias',status:'current'},
  {id:'dev-recepcion',name:'Recepción',os:'Windows',app:'Nightingale 0.1',last:'Hoy, 08:35',authorized:'2026-09-20 · Elena Arias',status:'active'},
  {id:'dev-tablet',name:'Tablet de Lucía',os:'iPadOS',app:'Nightingale 0.1',last:'Ayer, 17:02',authorized:'2026-09-20 · Elena Arias',status:'revoking'},
  {id:'dev-portatil',name:'Portátil antiguo',os:'Windows',app:'Nightingale 0.1',last:'28 de septiembre, 19:40',authorized:'2026-09-02 · Elena Arias',status:'revoked'}
];
function ensureService(name){if(serviceCatalog.some(s=>s.name===name))return;const id='s'+(serviceCatalog.length+1);serviceCatalog.push({id,name,baseDuration:30,active:true});serviceOffers.push({id:'o_'+id,serviceId:id,override:null,active:true});}
const professionals = [{id:'elena',name:'Elena Arias'},{id:'martin',name:'Martín Vega'}];
const responsibleContacts=[
  {id:'c1',name:'Laura Rivas',phoneCode:'+58',phoneNumber:'412 000 2001',relationship:'Madre',role:'Contacto y decisiones administrativas',active:true,since:'2026-04-10',representation:'declared'},
  {id:'c2',name:'Tomás Rivas',phoneCode:'+58',phoneNumber:'414 000 3002',relationship:'Padre',role:'Solo contacto',active:false,since:'2025-09-01',until:'2026-06-30',representation:'none'}
];
appointmentSeed.forEach(a=>{a.professionalId||='elena';});
// Teléfono del contrato: código y número son una pareja; ambos presentes o ambos vacíos.
const phoneText = p => p.phoneCode&&p.phoneNumber ? p.phoneCode+' '+p.phoneNumber : p.phoneCode&&!p.phoneNumber ? p.phoneCode+' · falta el número' : !p.phoneCode&&p.phoneNumber ? 'Falta el código · '+p.phoneNumber : '';
const phonePairInvalid = p => (p.phoneCode&&!p.phoneNumber)||(!p.phoneCode&&p.phoneNumber);
const docTypeLabel = {national_id:'Documento nacional',passport:'Pasaporte'};
const identifierText = p => p.identifier?`${p.identifier.country==='VE'?'Venezuela':p.identifier.country} · ${docTypeLabel[p.identifier.type]||p.identifier.type} · ${p.identifier.number}`:'Sin documento registrado';
const verificationText = status => ({declared:'Declarado',verified:'Verificado',invalidated:'Invalidado'})[status]||'';
const completionBadge = p => p.completion==='complete'?'<span class="badge">Ficha completa</span>':'<span class="badge attention">Por completar</span>';
const patientStatusBadge = p => p.active===false?'<span class="badge outline">Inactivo</span>':'<span class="badge">Activo</span>';
const patientMissing = p => {const missing=[],pairOk=!!(p.phoneCode&&p.phoneNumber),responsibleOk=(p.responsibleIds||[]).some(rid=>{const c=responsibleContacts.find(c=>c.id===rid);return c&&c.active!==false&&c.phoneCode&&c.phoneNumber;});if(!p.needsResponsible&&!p.identifier)missing.push('documento propio');if(p.needsResponsible&&!p.birthDate)missing.push('fecha de nacimiento');if(p.needsResponsible&&!responsibleOk)missing.push('responsable vigente con teléfono');if(!pairOk&&!responsibleOk)missing.push('teléfono completo o responsable');return missing;};
const representationLabel = {declared:'Declarada',documented:'Documentada',none:'Sin constancia'};
const responsibleVigencia = c => c.active===false?`Vigencia finalizada${c.until?' el '+longDate(c.until):''}`:`Vigente desde ${c.since?longDate(c.since):'fecha no registrada'}`;
const responsibleActivity = c => {let last=null,who='';for(const p of state.patients.filter(p=>(p.responsibleIds||[]).includes(c.id)))for(const a of state.appointments.filter(a=>a.patient===p.id)){if(!last||(a.date+a.start)>(last.date+last.start)){last=a;who=p.name;}}return last?`Última cita de ${who}: ${fmt(last.date)}, ${last.start}`:'Sin actividad conocida en este equipo';};
// Fichas completas que perderían sus requisitos si este responsable dejara de estar vigente.
const responsibleConflicts = c => state.patients.filter(p=>{if(!(p.responsibleIds||[]).includes(c.id)||p.completion!=='complete')return false;const others=(p.responsibleIds||[]).some(rid=>{if(rid===c.id)return false;const rc=responsibleContacts.find(x=>x.id===rid);return rc&&rc.active!==false&&rc.phoneCode&&rc.phoneNumber;});return !others&&!p.phoneCode&&!p.phoneNumber;});
const requestSeed=[
  {id:'r1',appointmentId:'a2',received:'2026-10-21T16:20',message:'No podré asistir ese día. ¿Podrían ofrecerme otra fecha?',status:'pending'},
  {id:'r2',appointmentId:'a5',received:'2026-10-22T08:10',message:'¿Me la pueden cambiar a otra fecha? Trabajo ese día.',status:'pending'},
  {id:'r3',appointmentId:'a17',received:'2026-10-20T11:05',message:'Prefiero por la tarde, si es posible.',status:'stale'},
  {id:'r4',appointmentId:'a16',received:'2026-10-19T09:40',message:'Necesito cambiar el día, avísenme por favor.',status:'partial'}
];
const commSeed=[
  {id:'m1',patient:'p1',appointmentId:'a1',type:'Recordatorio de cita',channel:'WhatsApp',date:'2026-10-21',time:'09:00',status:'Entregado',text:'Resultado recibido: entregado. La entrega no confirma asistencia.'},
  {id:'m2',patient:'p2',appointmentId:'a2',type:'Recordatorio de cita',channel:'WhatsApp',date:'2026-10-21',time:'09:30',status:'Entregado',text:'Resultado recibido: entregado. La entrega no confirma asistencia.'},
  {id:'m3',patient:'p4',appointmentId:'a4',type:'Confirmación de asistencia',channel:'WhatsApp',date:'2026-10-21',time:'17:10',status:'Respondido',text:'Respuesta recibida: el paciente confirmó asistencia. La respuesta queda registrada con la comunicación; el estado de la cita lo cambia el personal al confirmarla, no este mensaje.'},
  {id:'m4',patient:'p5',appointmentId:'a5',type:'Recordatorio de cita',channel:'WhatsApp',date:'2026-10-22',time:'08:00',status:'Programado',text:'Envío programado para la mañana anterior a la cita. Todavía no hay resultado de entrega.'},
  {id:'m5',patient:'p6',appointmentId:'a6',type:'Recordatorio de cita',channel:'WhatsApp',date:'2026-10-22',time:'08:30',status:'Revisar teléfono',text:'Fallo conocido: el número no pudo validarse en este ejemplo. No ofrecemos un reenvío automático; revisa el contacto antes de cualquier otro intento.'},
  {id:'m6',patient:'p3',appointmentId:'a3',type:'Recordatorio de cita',channel:'WhatsApp',date:'2026-10-22',time:'08:30',status:'Resultado desconocido',text:'Sin confirmación de entrega en este ejemplo (tiempo de espera agotado). No reenviamos a ciegas: revisa el contacto o espera la actualización del resultado.'}
];
const state = {patients:structuredClone(patientSeed),appointments:structuredClone(appointmentSeed),date:TODAY,query:'',filter:'',scenario:'normal',waitlist:[{id:'w1',patient:'p5',service:'Control de seguimiento',preference:'Mañanas · lunes a viernes',since:'2026-10-19'},{id:'w2',patient:'p7',service:'Primera consulta',preference:'Tardes · martes o jueves',since:'2026-10-20'},{id:'w3',patient:'p8',service:'Control de seguimiento',preference:'Cualquier horario',since:'2026-10-21'}],requests:structuredClone(requestSeed),messages:structuredClone(commSeed),commFilter:'',metricPeriod:'month',reviewDismissed:[],lastSearch:'',hubStatus:'Programada'};
const getPatient = id => state.patients.find(p=>p.id===id)||state.patients[0];
const getAppointment = id => state.appointments.find(a=>a.id===id)||state.appointments[0];
let route = '';
let renderedHash = '';
let activeDraft = null;
let pendingNavigation = null;
let toastTimer;

state.viewerRole='owner';
state.professionalCount=1;
state.professionalFilter='';
const showProfessional = () => state.viewerRole!=='owner'||state.professionalCount!==1;
const professionalName = id => professionals.find(p=>p.id===id)?.name||'Elena Arias';
const matchesProfessional = a => professionals.slice(0,state.professionalCount).some(p=>p.id===a.professionalId)&&(!state.professionalFilter||a.professionalId===state.professionalFilter);
state.agendaView='agenda';
state.detailOrigin='agenda';
state.agendaMode='ready';
state.serviceFilter='';
state.appointmentQuery='';
state.includeCancelled=false;
state.listPeriod='upcoming';
state.listLimit=8;
state.agendaScroll={top:0,left:0,weekTop:0};
state.canEditAppointments=true;
state.exampleNow='2026-10-22T10:20';
state.account={practice:'Consulta de Elena Arias',professional:'Dra. Elena Arias',specialty:'Medicina general',place:{name:'Centro Médico Los Olivos',room:'Consultorio 204',street:'Av. Los Olivos, Torre B',city:'Caracas',state:'Distrito Capital'}};
state.localPrefs={hourFormat:'24 horas'};
const accountLocation = () => {const p=state.account?.place;return p?[p.name,p.room].filter(Boolean).join(' · '):'Centro Médico Los Olivos · Consultorio 204';};
const terminalStatuses=['Cancelada','Atendida','Ausencia'];
const staffName = () => state.viewerRole==='assistant'?'Lucía Torres':'Elena Arias';
const canEditAppointments = () => state.canEditAppointments&&state.agendaMode!=='no-permission';
const appointmentSnapshot = a => ({date:a.date,start:a.start,end:a.end,status:a.status});
state.appointments.forEach(a=>{a.accepted=appointmentSnapshot(a);a.location='Centro Médico Los Olivos · Consultorio 204';a.professionalSnapshot=professionalName(a.professionalId);a.history=[{label:'Cita registrada',when:'2026-10-18T16:20',actor:'Elena Arias',interval:`${fmt(a.date)} · ${a.start} – ${a.end}`}];if(terminalStatuses.includes(a.status)||a.status==='Confirmada')a.history.push({label:`Estado: ${a.status}`,when:a.status==='Confirmada'?'2026-10-21T17:10':a.date+'T'+a.end,actor:'Elena Arias',interval:''});a.localActivity=[];});
function actionAllowed(a,action){
  if(!a||!canEditAppointments()||terminalStatuses.includes(a.status))return false;
  const now=new Date(state.exampleNow+':00-04:00').getTime(),start=new Date(a.date+'T'+a.start+':00-04:00').getTime();
  return action==='confirm'?a.status==='Programada':action==='complete'?now>=start:action==='absence'?now>=start+15*60000:true;
}
function recordAppointmentChange(a,label){
  const entry={label,when:state.exampleNow,actor:staffName(),interval:`${fmt(a.date)} · ${a.start} – ${a.end}`};
  if(state.scenario==='normal'){a.history=[...a.history,...a.localActivity.slice().reverse(),entry];a.localActivity=[];a.accepted=appointmentSnapshot(a);a.pendingAction=false;}
  else{a.localActivity.unshift(entry);a.pendingAction=true;}
}
function changeAppointment(id,action){
  const a=state.appointments.find(a=>a.id===id);
  if(!actionAllowed(a,action)){toast('Esta acción no está disponible para la cita.');return;}
  a.status={confirm:'Confirmada',complete:'Atendida',absence:'Ausencia',cancel:'Cancelada'}[action];
  recordAppointmentChange(a,{confirm:'Asistencia confirmada',complete:'Atención registrada',absence:'Ausencia registrada',cancel:'Cancelación guardada'}[action]);
  if(document.getElementById('dialog').open)closeDialog();render();toast('Cambios guardados en el ejemplo.');
}
function saveAppointment(form,values){
  if(form.dataset.submitted)return;
  if(!canEditAppointments())return errorMessage('No tienes permiso para modificar las citas. Tus datos se conservan.');
  const original=form.dataset.id?state.appointments.find(a=>a.id===form.dataset.id):null;
  if(original&&terminalStatuses.includes(original.status))return errorMessage('Esta cita ya finalizó y no se puede reprogramar.');
  const patient=state.patients.find(p=>p.id===(original?.patient||values.patient));
  if(!patient)return errorMessage('Elige un paciente para la cita.');
  if(patient.active===false||patient.completion!=='complete')return errorMessage('Completa la ficha administrativa del paciente antes de citarlo. Tus campos se conservan.');
  const responsibleId=original?.responsibleContactId||values.responsibleContactId||'';
  if(patient.needsResponsible&&!responsibleContacts.some(c=>c.id===responsibleId&&c.active!==false&&patient.responsibleIds.includes(c.id)&&c.phoneCode&&c.phoneNumber))return errorMessage('Selecciona un responsable vigente con teléfono.');
  const professionalId=original?.professionalId||values.professionalId;
  const date=values.date,start=values.start;
  const duration=original?minutes(original.end)-minutes(original.start):minutes(values.end)-minutes(start);
  const end=original?clock(minutes(start)+duration):values.end;
  if(!start||!end||!Number.isFinite(duration)||duration<=0||end<=start)return errorMessage('Revisa el inicio y el fin del intervalo.');
  if(!covered(date)||professionalId!=='elena'||['loading','read-error'].includes(state.agendaMode))return errorMessage('No hay cobertura suficiente para comprobar ese horario. Tus datos se conservan.');
  const dayWindows=windowsForDate(date);
  if(!dayWindows.length)return errorMessage('Ese día está cerrado según el horario de la consulta. Tus datos se conservan.');
  if(!dayWindows.some(w=>minutes(start)>=w.start&&minutes(end)<=w.end))return errorMessage(`Elige un intervalo dentro del horario: ${windowsText(date)}. Tus datos se conservan.`);
  if(occupiedIntervals(date,professionalId,original?.id).some(a=>a.start<end&&a.end>start))return errorMessage('Ese intervalo coincide con una cita o un bloqueo. Tus datos se conservan.');
  if(state.scenario==='conflict')return errorMessage('El horario cambió en otro equipo. Conservamos la propuesta para que elijas otro intervalo.');
  form.dataset.submitted='true';
  let saved=original;
  if(original){original.previous=appointmentSnapshot(original);Object.assign(original,{date,start,end,status:'Programada'});recordAppointmentChange(original,'Nuevo horario guardado');}
  else{saved={id:'a'+(state.appointments.length+1),patient:patient.id,professionalId,service:values.service,date,start,end,responsibleContactId:responsibleId,status:'Programada',location:accountLocation(),professionalSnapshot:professionalName(professionalId),history:[],localActivity:[],accepted:null,created:true};state.appointments.push(saved);recordAppointmentChange(saved,'Cita guardada');}
  state.date=date;form.removeAttribute('data-dirty');activeDraft=null;
  const solicitudId=new URLSearchParams(location.hash.split('?')[1]||'').get('solicitud');
  if(solicitudId){const req=state.requests.find(r=>r.id===solicitudId);if(req)req.status='resolved';}
  go('cita/'+saved.id);toast('Cita guardada en el ejemplo.');
}
function occupiedIntervals(date,professionalId,ignoreId=''){
  const intervals=[];
  for(const a of state.appointments){if(a.id===ignoreId||a.professionalId!==professionalId)continue;if(a.status!=='Cancelada'&&a.date===date)intervals.push(a);if(a.pendingAction&&a.accepted&&a.accepted.status!=='Cancelada'&&a.accepted.date===date&&(a.status==='Cancelada'||a.accepted.date!==a.date||a.accepted.start!==a.start||a.accepted.end!==a.end))intervals.push(a.accepted);}
  return intervals.concat(scheduleBlocks.filter(b=>b.date===date&&b.professionalId===professionalId));
}
const coverage={from:'2026-10-19',to:'2026-10-30'};
const scheduleRules=[
  {id:'r1',days:[1,2,3,4,5],intervals:[{start:'09:00',end:'12:00'},{start:'14:00',end:'17:00'}],validFrom:'2026-10-19',validTo:'2026-12-31'}
];
const scheduleExceptions=[
  {id:'e1',date:'2026-10-26',type:'closed',intervals:[],label:'Mantenimiento del consultorio'}
];
const scheduleBlocks=[{id:'b1',date:TODAY,start:'16:00',end:'16:30',professionalId:'elena',global:true,label:'Gestiones personales'}];
const dayNames=['Domingo','Lunes','Martes','Miércoles','Jueves','Viernes','Sábado'];
const exceptionsForDate = date => scheduleExceptions.filter(e=>e.date===date);
// Ventanas de atención en minutos: la excepción del día sustituye a las reglas; el cierre deja el día vacío.
const windowsForDate = date => {const ex=exceptionsForDate(date)[0];if(ex)return ex.type==='closed'?[]:ex.intervals.map(i=>({start:minutes(i.start),end:minutes(i.end)}));const weekday=new Date(date+'T12:00:00Z').getUTCDay();return scheduleRules.filter(r=>r.days.includes(weekday)&&date>=r.validFrom&&date<=r.validTo).flatMap(r=>r.intervals.map(i=>({start:minutes(i.start),end:minutes(i.end)})));};
const windowsText = date => windowsForDate(date).map(w=>clock(w.start)+'–'+clock(w.end)).join(' o ');
const requestStatusMeta = {pending:{label:'Solicitud de cambio',cls:'attention'},partial:{label:'Conciliación pendiente',cls:'attention'},stale:{label:'Desactualizada',cls:''},resolved:{label:'Resuelta',cls:''}};
const requestBadge = r => `<span class="badge ${requestStatusMeta[r.status]?.cls||''}">${requestStatusMeta[r.status]?.label||esc(r.status)}</span>`;
const requestStatusText = r => ({pending:'Cita sin modificar: elegir un horario no cambia la cita hasta que guardes el cambio.',partial:'El cambio de cita ya fue aceptado, pero esta solicitud sigue abierta: falta actualizarla en el ejemplo. No requiere una nueva reserva.',stale:'Solicitud desactualizada: la cita ya tiene otro horario, así que esta petición ya no aplica. No se duplica ningún cambio.',resolved:'Cambio registrado y solicitud actualizada en el ejemplo.'})[r.status]||'';
const ageText = iso => {const then=new Date(iso+':00-04:00').getTime(),now=new Date(state.exampleNow+':00-04:00').getTime();const day=86400000;const days=Math.floor((new Date(now).setHours(0,0,0,0)-new Date(then).setHours(0,0,0,0))/day);const time=iso.slice(11,16);return days<=0?`hoy, ${time}`:days===1?`ayer, ${time}`:`hace ${days} días, ${time}`;};
const minutes = time => Number(time.slice(0,2))*60+Number(time.slice(3,5));
const clock = n => String(Math.floor(n/60)).padStart(2,'0')+':'+String(n%60).padStart(2,'0');
const referenceDuration = service => {if(state.setup?.reviewed&&state.setup.service===service)return Number(state.setup.duration);const c=serviceCatalog.find(s=>s.name===service),o=c&&offerByServiceId(c.id);return (o&&o.override)||(c&&c.baseDuration)||30;};
const covered = date => date>=coverage.from&&date<=coverage.to;
function shiftedDate(date,days){const d=new Date(date+'T12:00:00Z');d.setUTCDate(d.getUTCDate()+days);return d.toISOString().slice(0,10);}
function weekDates(){const weekday=new Date(state.date+'T12:00:00Z').getUTCDay();const monday=shiftedDate(state.date,-((weekday+6)%7));return Array.from({length:5},(_,i)=>shiftedDate(monday,i));}
function professionalControl(){
  return showProfessional()?`<select id="professional-filter" aria-label="Filtrar por profesional"><option value="">Todos los profesionales</option>${professionals.slice(0,state.professionalCount).map(p=>`<option value="${p.id}" ${state.professionalFilter===p.id?'selected':''}>${esc(p.name)}</option>`).join('')}</select>`:'';
}

const screens = [
  ['Acceso y preparación',[['acceso','Acceso al escritorio'],['acceso/navegador','Acceso en el navegador'],['acceso/cuentas','Seleccionar consulta'],['acceso/equipo','Autorizar equipo'],['acceso/preparando','Preparación inicial pendiente'],['acceso/local','Acceso local permitido'],['acceso/sesion-vencida','Sesión remota vencida'],['acceso/denegado','Acceso revocado'],['acceso/conexion','Conexión necesaria'],['acceso/servicio','Servicio de acceso no disponible'],['preparacion','Preparación de la consulta'],['preparacion/lista','Resultado de preparación']]],
  ['Agenda y citas',[['agenda','Agenda del día'],['agenda/semana','Agenda semanal'],['citas','Lista de citas'],['cita/a1','Detalle de cita'],['cita/nueva','Nueva cita'],['cita/a1/reprogramar','Reprogramar cita'],['cita/a16/reprogramar','Reprogramar cita de 45 minutos'],['cita/a10','Cita atendida'],['cita/a11','Cita cancelada'],['cita/a19','Ausencia registrada']]],
  ['Pacientes',[['pacientes','Directorio de pacientes'],['paciente/p1','Ficha del paciente'],['paciente/p9','Ficha de menor con responsable'],['paciente/p10','Ficha incompleta · borrador'],['paciente/p11','Pacientes homónimos'],['pacientes/nuevo','Registrar paciente']]],
  ['Seguimiento',[['solicitudes','Solicitudes del paciente'],['solicitud/r1','Revisar una solicitud'],['lista-espera','Lista de espera'],['comunicaciones','Comunicaciones'],['indicadores','Indicadores']]],
  ['Configuración',[['configuracion/cuenta','Cuenta profesional'],['configuracion/servicios','Servicios'],['configuracion/horarios','Horarios y bloqueos'],['configuracion/comunicaciones','Política de comunicaciones'],['configuracion/equipo','Usuarios y permisos'],['configuracion/dispositivos','Dispositivos']]],
  ['Excepciones y ayuda',[['actividad','Por revisar'],['conflicto/a3','Conflicto de horario'],['diagnostico','Diagnóstico de desarrollo']]],
  ['Patient Hub',[['hub/cita','Cita del paciente'],['hub/confirmada','Confirmación recibida'],['hub/cancelar','Cancelar la cita'],['hub/cancelada','Cita cancelada'],['hub/cambiar','Solicitar cambio'],['hub/solicitud','Solicitud recibida'],['hub/cita-modificada','Cita modificada'],['hub/enlace-vencido','Enlace vencido o revocado'],['hub/no-disponible','Resultado por comprobar'],['hub/servicio','Servicio no disponible']]]
];
window.prototypeScreens = screens.flatMap(([group,items])=>items.map(([path,title])=>({group,path,title})));

function brand(target='agenda'){return `<a href="#/${target}" class="brand" aria-label="Nightingale"><span class="brand-mark" aria-hidden="true">N</span>Nightingale</a>`;}
function prototypeBar(){return `<footer class="prototype-bar"><span>Prototipo de diseño · Datos ficticios</span><div class="tools">${link('Ver todas las pantallas','pantallas')}${button('Colores propuestos','palette')}${button('Probar un escenario','scenarios')}${button('Restablecer','reset')}</div></footer>`;}
function navItem(text,path,ico,active,count=''){return `<a href="${href(path)}" ${active?'aria-current="page"':''}>${icon(ico)}<span>${text}</span>${count?`<span class="count">${count}</span>`:''}</a>`;}
function shell(content,title,section='agenda',crumb=''){
  const parentRoutes={agenda:'agenda',pacientes:'pacientes',solicitudes:'solicitudes',espera:'lista-espera',comunicaciones:'comunicaciones',indicadores:'indicadores',configuracion:'configuracion/cuenta',revision:'actividad',ayuda:'diagnostico'};
  const parent=parentRoutes[section]||'agenda';
  const breadcrumb=crumb?link(esc(title),parent)+'<span class="slash">/</span><span aria-current="page">'+esc(crumb)+'</span>':'<span aria-current="page">'+esc(title)+'</span>';
  const assistant=state.viewerRole==='assistant';
  return   `<div class="app-shell"><aside class="sidebar" id="sidebar" aria-label="Navegación de la consulta">
    <div class="sidebar-brand">${brand()}${button(icon('close'),'close-menu','quiet icon mobile-menu','aria-label="Cerrar navegación"')}</div>
    <div class="practice"><span>${esc(state.account?.practice||state.setup?.practice||'Consulta de Elena Arias')}</span></div>
    <nav class="nav" aria-label="Trabajo diario">${navItem('Agenda','agenda','calendar',section==='agenda')}${navItem('Pacientes','pacientes','people',section==='pacientes')}${navItem('Solicitudes','solicitudes','inbox',section==='solicitudes',String(state.requests.filter(r=>r.status==='pending').length||''))}${navItem('Lista de espera','lista-espera','clock',section==='espera')}</nav>
    <div><div class="nav-label">Seguimiento</div><nav class="nav" aria-label="Seguimiento">${navItem('Comunicaciones','comunicaciones','message',section==='comunicaciones')}${navItem('Indicadores','indicadores','chart',section==='indicadores')}</nav></div>
    <div class="sidebar-bottom"><nav class="nav" aria-label="Preferencias">${reviewItems().length?navItem('Por revisar','actividad','info',section==='revision',String(reviewItems().length)):''}${navItem('Configuración','configuracion/cuenta','settings',section==='configuracion')}</nav>
    <a class="profile" href="#/configuracion/cuenta">${avatar(assistant?'Lucía Torres':'Elena Arias',assistant?'staff-lucia':'staff-elena')}<span><strong class="small">${assistant?'Lucía Torres':'Elena Arias'}</strong><span class="block small subtle">${assistant?'Asistente':'Médica · Dueña'}</span></span></a></div>
  </aside><div class="workspace"><header class="topbar"><div class="breadcrumb">${button(icon('menu'),'menu','quiet icon mobile-menu','aria-label="Abrir menú" aria-controls="sidebar" aria-expanded="false"')}${icon(section==='pacientes'?'people':section==='solicitudes'?'inbox':section==='configuracion'?'settings':'calendar')}${breadcrumb}</div><div class="top-actions">${button(`<span>${icon('search')} <span class="search-copy">Buscar en la consulta</span></span><kbd>Ctrl K</kbd>`,'search','search-trigger','aria-label="Buscar paciente o cita"')}${button(icon('help'),'help','quiet icon','aria-label="Ayuda y pantallas"')}</div></header><main id="main">${scenarioNotice()}${content}</main></div></div>${prototypeBar()}`;
}
function head(title,subtitle='',actions=''){return `<div class="page-head"><div><h1>${title}</h1>${subtitle?`<p>${subtitle}</p>`:''}</div>${actions?`<div class="head-actions">${actions}</div>`:''}</div>`;}
function tabs(items,active){return `<nav class="tabs" aria-label="Vistas">${items.map(([name,path])=>`<a href="${href(path)}" ${path===active?'aria-current="page"':''}>${name}</a>`).join('')}</nav>`;}
function notice(title,text,action=''){return `<div class="notice">${icon('info')}<div><strong>${title}</strong>${text?`<p>${text}</p>`:''}</div>${action}</div>`;}
function scenarioNotice(){
  if(state.scenario==='conflict'&&['agenda','agenda/semana','citas','cita/a3'].includes(route))return notice('Una cita necesita tu revisión.','Otro equipo reservó el horario que habías elegido.',link('Revisar','conflicto/a3','button'));
  return '';
}
function empty(title,text,action='',ico='calendar'){return `<div class="empty">${icon(ico)}<h2>${title}</h2><p>${text}</p>${action}</div>`;}
function options(values,selected){return values.map(v=>`<option${v===selected?' selected':''}>${esc(v)}</option>`).join('');}

function appointmentRows(records,compact=false){
  const professional=showProfessional();
  return `<table class="data-table"><thead><tr><th><span class="column-label">${icon('clock')} Horario</span></th><th><span class="column-label">${icon('people')} Paciente</span></th>${compact?'':`<th class="service-col"><span class="column-label">${icon('grid')} Servicio</span></th>`}${professional?'<th>Profesional</th>':''}<th>Estado</th><th class="row-actions"><span class="sr-only">Acciones</span></th></tr></thead><tbody>${records.map(a=>`<tr><td class="time">${esc(a.start)} <span class="subtle">– ${esc(a.end)}</span>${state.agendaView==='citas'||route.startsWith('paciente/')?`<span class="block small subtle">${fmt(a.date)}</span>`:''}</td><td><a class="link-row" href="#/cita/${a.id}">${compact?'':avatar(getPatient(a.patient).name,a.patient)}<span class="name">${esc(getPatient(a.patient).name)}</span></a>${state.scenario==='conflict'&&a.id==='a3'?'<a class="review-link small" href="#/conflicto/a3">Revisar horario</a>':''}</td>${compact?'':`<td class="service-col muted">${esc(a.service)}</td>`}${professional?`<td class="muted">${esc(professionalName(a.professionalId))}</td>`:''}<td>${badge(a.status)}</td><td class="row-actions">${button(icon('more'),`appointment-menu:${a.id}`,'quiet icon',`aria-label="Acciones de la cita de ${esc(getPatient(a.patient).name)}"`)}</td></tr>`).join('')}</tbody></table>`;
}
function agendaToolbar(){
  const weekly=state.agendaView==='agenda/semana';
  return `<div class="toolbar"><div class="toolbar-group"><div class="segmented">${button(icon('left'),`date:${weekly?-7:-1}`,'',`aria-label="${weekly?'Semana':'Día'} anterior"`)}${button(icon('right'),`date:${weekly?7:1}`,'',`aria-label="${weekly?'Semana':'Día'} siguiente"`)}</div>${button('Hoy','today')}<input id="agenda-date" type="date" value="${state.date}" aria-label="Fecha de la agenda"></div><div class="toolbar-group"><div class="search">${icon('search')}<input id="appointment-search" type="search" value="${esc(state.appointmentQuery)}" placeholder="Buscar paciente…" aria-label="Buscar citas por paciente"></div><select id="appointment-filter" aria-label="Filtrar por estado"><option value="">Todos los estados</option>${options(['Programada','Confirmada','Atendida','Ausencia','Cancelada'],state.filter)}</select>${professionalControl()}${button(icon('filter')+' Filtros','filters','quiet')}</div></div>`;
}
function activeFilters(){
  const values=[state.appointmentQuery&&`Paciente: ${state.appointmentQuery}`,state.filter&&`Estado: ${state.filter}`,state.serviceFilter&&`Servicio: ${state.serviceFilter}`,state.professionalFilter&&`Profesional: ${professionalName(state.professionalFilter)}`,state.includeCancelled&&'Incluye canceladas'].filter(Boolean);
  return values.length?`<div class="filter-summary"><span>${values.map(esc).join(' · ')}</span>${button('Limpiar filtros','clear-filter','quiet')}</div>`:'';
}
function visibleAppointments(records,history=false){
  return records.filter(a=>matchesProfessional(a)&&(!state.filter||a.status===state.filter)&&(!state.serviceFilter||a.service===state.serviceFilter)&&getPatient(a.patient).name.toLocaleLowerCase('es').includes(state.appointmentQuery.toLocaleLowerCase('es'))&&(a.status!=='Cancelada'||history||state.includeCancelled||state.filter==='Cancelada'));
}
function availableSlot(date,professionalId=state.professionalFilter||'elena',duration=30,ignoreId=''){
  if(!covered(date)||professionalId!=='elena')return null;
  const windows=windowsForDate(date);
  if(!windows.length)return null;
  const occupied=occupiedIntervals(date,professionalId,ignoreId);
  for(const w of windows)for(let start=w.start;start+duration<=w.end;start+=15){const end=start+duration;if(!occupied.some(a=>minutes(a.start)<end&&minutes(a.end)>start))return {start:clock(start),end:clock(end)};}
  return null;
}
function agendaState(view){
  if(state.agendaMode==='loading')return `<div class="agenda-loading" role="status" aria-busy="true"><p>Cargando la agenda…</p>${Array.from({length:5},()=>'<div class="skeleton-row"></div>').join('')}</div>`;
  if(state.agendaMode==='read-error')return empty('No pudimos leer la agenda','Tus datos guardados se conservan. Vuelve a cargar esta vista.',button('Volver a cargar','agenda-retry','primary'),'info');
  if(state.agendaMode==='no-permission')return empty('No tienes permiso para ver esta agenda','Revisa el acceso de tu cuenta. El trabajo guardado se conserva.',link('Revisar acceso','acceso/denegado','button'),'shield');
  if(view==='agenda'&&!covered(state.date))return empty('Esta fecha no está disponible en este equipo',`La vista local cubre del ${fmt(coverage.from)} al ${fmt(coverage.to)}. No podemos mostrar esta fecha como si estuviera vacía.`,button('Volver a hoy','today'),'calendar');
  return '';
}
function agendaPage(view,content,detailId=''){
  const dates=weekDates();
  const title=view==='citas'?'Lista de citas':view==='agenda/semana'?`${fmt(dates[0])} – ${fmt(dates[4])}`:state.date===TODAY?'Hoy, jueves 22':titleCase(fullDate(state.date));
  const status=agendaState(view);
  const subtitle=view==='citas'?`Citas disponibles en este equipo · ${fmt(coverage.from)} – ${fmt(coverage.to)}`:`${state.date.slice(0,4)} · Hora de Caracas`;
  const action=detailId||!canEditAppointments()||['loading','read-error','no-permission'].includes(state.agendaMode)?'':link(icon('plus')+' Nueva cita','cita/nueva','button primary');
  const progress=state.agendaMode==='refreshing'?'<p class="agenda-update" role="status">Actualizando la agenda… Puedes seguir consultando los datos visibles.</p>':state.agendaMode==='stale'?notice('Esta vista puede estar desactualizada.','Última información conocida: 22 de octubre, 08:40. Las acciones conservan su validación.'):'';
  let body=status||progress+content;
  return shell(head(title,subtitle,action)+tabs([['Día','agenda'],['Semana','agenda/semana'],['Lista de citas','citas']],view)+agendaToolbar()+activeFilters()+body,'Agenda','agenda',detailId?'Detalle de cita':'');
}
function agenda(detailId=''){
  const records=visibleAppointments(state.appointments.filter(a=>a.date===state.date)).sort((a,b)=>a.start.localeCompare(b.start));
  const hasFilters=state.filter||state.serviceFilter||state.professionalFilter||state.appointmentQuery||state.includeCancelled;
  const slot=state.professionalCount>1&&!state.professionalFilter?null:availableSlot(state.date);
  const closed=[0,6].includes(new Date(state.date+'T12:00:00Z').getUTCDay());
  const body=records.length?`<div class="day-caption"><span>${records.length} citas ${hasFilters?'en esta vista':'en el día'}</span><span>Consulta presencial · ${esc(state.account?.place?.room||'Consultorio 204')}</span></div><div class="table-wrap">${appointmentRows(records)}</div>`:hasFilters?empty('No hay citas que coincidan','Prueba con otro filtro. Los filtros no cambian la disponibilidad.',button('Limpiar filtros','clear-filter')):empty('No hay citas para este día',closed?'No hay horario de atención este día.':'La fecha está disponible en este equipo y todavía no tiene citas.',closed||!canEditAppointments()?'':link('Nueva cita','cita/nueva','button'));
  const blocks=covered(state.date)&&!closed?`<details class="day-blocks"><summary>Horario y bloqueos del día</summary><div class="list-item"><span>Horario de atención</span><span>${windowsText(state.date)||'Día cerrado'}</span></div>${exceptionsForDate(state.date).map(ex=>`<div class="list-item"><span>${ex.type==='closed'?'Cierre del día':'Reemplazo de horario'}${ex.label?': '+esc(ex.label):''}</span><span>${ex.type==='closed'?'Sin atención':ex.intervals.map(i=>i.start+' – '+i.end).join(' · ')}</span></div>`).join('')}${scheduleBlocks.filter(b=>b.date===state.date&&matchesProfessional(b)).map(b=>`<div class="list-item"><span>${b.global&&state.viewerRole==='assistant'?'Bloqueo de agenda':esc(b.label)}</span><span>${b.start} – ${b.end}</span></div>`).join('')}</details>`:'';
  const availability=slot&&canEditAppointments()?`<div class="availability"><span>Primer intervalo disponible</span>${link(icon('plus')+' '+slot.start+' – '+slot.end,`cita/nueva?hora=${slot.start}&fin=${slot.end}&fecha=${state.date}`,'button')}</div>`:'';
  return agendaPage('agenda',body+availability+blocks,detailId);
}

function appointmentDetail(a){
  const p=getPatient(a.patient),done=terminalStatuses.includes(a.status),editable=canEditAppointments();
  const responsible=responsibleContacts.find(c=>c.id===a.responsibleContactId);
  const facts=[['Profesional',a.professionalSnapshot||professionalName(a.professionalId),'people'],['Lugar de atención',a.location||'Centro Médico Los Olivos · Consultorio 204','pin'],['Contacto',responsible?phoneText(responsible)||responsible.name:phoneText(p)||'No registrado','phone']];
  if(responsible)facts.push(['Responsable',responsible.name+' · '+responsible.relationship,'people']);
  let actions='';
  if(editable&&!done){
    const primary=actionAllowed(a,'complete')?button('Registrar atención',`complete:${a.id}`,'primary'):actionAllowed(a,'confirm')?button('Confirmar asistencia',`confirm:${a.id}`,'primary'):'';
    actions=`<div class="detail-actions">${primary}${link('Reprogramar',`cita/${a.id}/reprogramar`,'button')}</div><div class="appointment-more"><button type="button" class="quiet" popovertarget="appointment-more-actions">Más acciones ${icon('down')}</button><div id="appointment-more-actions" class="action-popover" popover>${actionAllowed(a,'confirm')&&actionAllowed(a,'complete')?button('Confirmar asistencia',`confirm:${a.id}`,'quiet'):''}${actionAllowed(a,'absence')?button('Registrar ausencia',`absence:${a.id}`,'quiet'):''}${button('Cancelar cita',`cancel:${a.id}`,'quiet')}</div></div>`;
  }
  const history=a.history||[];
  const local=a.localActivity||[];
  const comms=state.messages.filter(m=>m.appointmentId===a.id);
  const commBlock=comms.length?comms.map(m=>`<p class="small">${esc(m.type)} ${badge(m.status)}</p><p class="field-help">${fmt(m.date)} · ${m.time}</p>`).join(''):'<p class="field-help">Sin comunicaciones registradas.</p>';
  const eventMarkup=event=>`<li><strong>${esc(event.label)}</strong><span class="block small muted">${esc(event.interval||'')}</span><small>${fmt(event.when.slice(0,10))} · ${event.when.slice(11,16)} · ${esc(event.actor)}</small></li>`;
  return `<div class="appointment-modal-head"><div class="appointment-person">${avatar(p.name,p.id)}<div><p class="eyebrow">Detalle de la cita</p><h2 id="appointment-title" tabindex="-1">${esc(p.name)}</h2><p class="appointment-service">${esc(a.service)}</p></div></div>${button(icon('close'),'close-appointment','quiet icon','aria-label="Cerrar detalle"')}</div><section class="appointment-time" aria-label="Fecha y horario"><div class="appointment-status">${badge(a.status)}<span class="small subtle">Hora de Caracas</span></div><p class="appointment-date">${icon('calendar')}${esc(titleCase(fullDate(a.date)))}, ${a.date.slice(0,4)}</p><p class="appointment-hours">${a.start}<span aria-label="a">–</span>${a.end}</p><p class="appointment-duration">${minutes(a.end)-minutes(a.start)} minutos de atención</p><div class="appointment-actions">${actions}</div></section>${state.scenario==='conflict'&&a.id==='a3'?notice('Revisa este horario','Tu propuesta se conserva.',link('Revisar','conflicto/a3','button')):''}<dl class="facts appointment-facts">${facts.map(([label,value,glyph])=>`<div class="fact"><dt>${icon(glyph)}<span>${label}</span></dt><dd>${esc(value)}</dd></div>`).join('')}</dl><div class="appointment-patient-link">${link('Ver ficha del paciente '+icon('arrow'),`paciente/${p.id}`,'button quiet')}</div>${!editable?'<p class="field-help">Solo tienes permiso de consulta.</p>':done?'<p class="field-help">Esta cita ha finalizado. Conservamos sus datos y su historia.</p>':''}<div class="appointment-communication section-line"><div>${icon('message')}<div><h3>Comunicaciones</h3>${commBlock}</div></div>${link('Ver comunicaciones '+icon('right'),'comunicaciones','button quiet')}</div><details class="appointment-history section-line"><summary>Actividad de la cita <span class="small subtle">${history.length+local.length}</span></summary>${local.length?`<ul class="history">${local.map(eventMarkup).join('')}</ul>`:''}<ul class="history">${history.slice().reverse().map(eventMarkup).join('')}</ul></details>`;
}

function placeWeekEvents(records){
  const placed=[];let group=[],ends=[],groupEnd=-1;
  const finish=()=>{group.forEach(item=>{item.columns=ends.length;});};
  for(const a of records.slice().sort((a,b)=>a.start.localeCompare(b.start)||a.end.localeCompare(b.end))){
    const start=minutes(a.start),end=minutes(a.end);
    if(start>=groupEnd){finish();group=[];ends=[];}
    let lane=ends.findIndex(value=>value<=start);if(lane<0)lane=ends.length;
    ends[lane]=end;groupEnd=Math.max(group.length?groupEnd:0,end);
    const item={appointment:a,lane,columns:1};group.push(item);placed.push(item);
  }
  finish();return placed;
}
function week(detailId=''){
  const dates=weekDates();
  let grid='<div class="week-corner"><span>Hora de<br>Caracas</span></div>'+dates.map(date=>`<button type="button" class="day-header ${date===TODAY?'today':''}" data-action="day:${date}" aria-label="Ver ${esc(fullDate(date))}"><span>${new Intl.DateTimeFormat('es-VE',{weekday:'short',timeZone:'UTC'}).format(new Date(date+'T12:00:00Z'))}</span><b>${Number(date.slice(-2))}</b></button>`).join('');
  grid+=`<div class="week-axis">${Array.from({length:9},(_,i)=>`<span class="week-hour" style="top:${i*96}px">${clock(540+i*60)}</span>`).join('')}</div>`;
  for(const date of dates){
    if(!covered(date)){grid+='<div class="week-day uncovered"><span>Fecha no disponible en este equipo</span></div>';continue;}
    const events=placeWeekEvents(visibleAppointments(state.appointments.filter(a=>a.date===date)));
    const bookings=events.map(({appointment:a,lane,columns})=>{
      const duration=minutes(a.end)-minutes(a.start),short=duration<30;
      const label=`${getPatient(a.patient).name}, ${fmt(a.date)}, ${a.start} a ${a.end}, ${a.service}, ${professionalName(a.professionalId)}, ${a.status}`;
      return `<a class="week-event ${statusClass(a.status)} ${short?'short-event':''}" href="#/cita/${a.id}" aria-label="${esc(label)}" title="${esc(label)}" style="top:${(minutes(a.start)-540)*1.6+2}px;height:${duration*1.6-4}px;left:calc(${lane*100/columns}% + 4px);width:calc(${100/columns}% - 8px)"><b>${short?a.start+' · ':''}${esc(getPatient(a.patient).name)}</b>${short?'':`<small>${a.start} – ${a.end} · ${esc(a.status)}</small>`}${duration>=45?`<span class="calendar-service">${esc(a.service)}</span>`:''}</a>`;
    }).join('');
    const windows=windowsForDate(date).map(w=>({start:Math.max(w.start,540),end:Math.min(w.end,1020)})).filter(w=>w.end>w.start);
    let closedBands=[];
    if(!windows.length)closedBands=[{start:540,end:1020}];
    else{if(windows[0].start>540)closedBands.push({start:540,end:windows[0].start});for(let i=1;i<windows.length;i++)closedBands.push({start:windows[i-1].end,end:windows[i].start});if(windows[windows.length-1].end<1020)closedBands.push({start:windows[windows.length-1].end,end:1020});}
    const blocks=closedBands.map(b=>`<div class="calendar-block" style="top:${(b.start-540)*1.6}px;height:${(b.end-b.start)*1.6}px"><span>Fuera de horario</span></div>`).join('')+scheduleBlocks.filter(b=>b.date===date&&matchesProfessional(b)&&minutes(b.end)>540&&minutes(b.start)<1020).map(b=>`<div class="calendar-block" style="top:${(Math.max(minutes(b.start),540)-540)*1.6}px;height:${(Math.min(minutes(b.end),1020)-Math.max(minutes(b.start),540))*1.6}px"><span>${b.global&&state.viewerRole==='assistant'?'Bloqueo de agenda':esc(b.label)}<br>${b.start} – ${b.end}</span></div>`).join('');
    grid+=`<div class="week-day" data-date="${date}">${'<div class="week-cell"></div>'.repeat(8)}${blocks}${bookings}</div>`;
  }
  const body=`<div class="day-caption"><span>Horario de atención · 09:00 – 17:00</span>${link('Revisar horario','configuracion/horarios','link')}</div><div class="week-scroll" tabindex="0" role="region" aria-label="Agenda semanal con desplazamiento"><div class="week">${grid}</div></div><p class="field-help">Abre una cita para ver todos sus datos. También puedes usar Lista de citas.</p>`;
  return agendaPage('agenda/semana',body,detailId);
}
function appointmentsList(detailId=''){
  const records=visibleAppointments(state.appointments.filter(a=>state.listPeriod==='upcoming'?a.date>=TODAY:state.listPeriod==='history'?a.date<TODAY:true),state.listPeriod==='history'||state.listPeriod==='all').sort((a,b)=>(a.date+a.start).localeCompare(b.date+b.start));
  const period=`<div class="list-period"><label>Periodo<select id="list-period"><option value="upcoming" ${state.listPeriod==='upcoming'?'selected':''}>Próximas citas</option><option value="history" ${state.listPeriod==='history'?'selected':''}>Historial disponible</option><option value="all" ${state.listPeriod==='all'?'selected':''}>Todo lo disponible</option></select></label><span class="small subtle">Solo información disponible en este equipo.</span></div>`;
  const body=records.length?`<div class="table-wrap">${appointmentRows(records.slice(0,state.listLimit),!!detailId)}</div><div class="day-caption"><span>${Math.min(state.listLimit,records.length)} de ${records.length} citas</span>${records.length>state.listLimit?button('Cargar más','more-appointments'):''}</div>`:empty('No hay citas que coincidan','Cambia el periodo o elimina los filtros.',button('Limpiar filtros','clear-filter'));
  return agendaPage('citas',period+body,detailId);
}

function bookingPatientOptions(draft){
  const query=(draft.patientSearch||'').toLocaleLowerCase('es');
  const patients=state.patients.filter(p=>p.active!==false&&(p.name+' '+phoneText(p)).toLocaleLowerCase('es').includes(query));
  return '<option value="">Elige un paciente</option>'+patients.map(p=>`<option value="${p.id}" ${draft.patient===p.id?'selected':''}>${esc(p.name)} · ${esc(phoneText(p)||(p.responsibleIds?.length?'Contacto responsable':'Sin teléfono'))}${p.completion!=='complete'?' · ficha incompleta':''}</option>`).join('');
}
function appointmentEditor(id=''){
  const original=id?state.appointments.find(a=>a.id===id):null;
  if(id&&(!original||terminalStatuses.includes(original.status)))return shell(head('Esta cita no se puede reprogramar')+empty('Consulta la cita y su historia','Las citas canceladas, atendidas o con ausencia no se pueden reabrir.',link('Volver a la cita',original?'cita/'+id:state.agendaView,'button')),'Agenda','agenda','Reprogramar');
  const query=new URLSearchParams(location.hash.split('?')[1]||'');
  const date=original?.date||query.get('fecha')||state.date;
  const slot=availableSlot(date);
  const start=original?.start||query.get('hora')||slot?.start||'09:00';
  const draft={patient:original?.patient||query.get('paciente')||'',patientSearch:'',professionalId:original?.professionalId||state.professionalFilter||'elena',service:original?.service||offeredNames()[0],date,start,end:original?.end||query.get('fin')||clock(minutes(start)+referenceDuration(offeredNames()[0])),responsibleContactId:original?.responsibleContactId||'',...(activeDraft||{})};
  const duration=original?minutes(original.end)-minutes(original.start):minutes(draft.end)-minutes(draft.start);
  if(original){draft.patient=original.patient;draft.professionalId=original.professionalId;draft.service=original.service;draft.responsibleContactId=original.responsibleContactId||'';draft.end=minutes(draft.start)+duration<1440?clock(minutes(draft.start)+duration):'';}
  const patient=state.patients.find(p=>p.id===draft.patient);
  const candidate=availableSlot(draft.date,draft.professionalId,original?duration:Math.max(1,duration),id);
  const identity=original?`<div class="booking-identity"><strong>${esc(getPatient(original.patient).name)}</strong><p>${esc(original.service)} · ${esc(original.professionalSnapshot)}${original.responsibleContactId?'<br>Responsable: '+esc(responsibleContacts.find(c=>c.id===original.responsibleContactId)?.name||'No disponible'):''}</p></div>`:`<fieldset class="form-section"><legend>Paciente</legend><div class="fields"><label>Buscar paciente<input id="booking-patient-search" type="search" name="patientSearch" value="${esc(draft.patientSearch)}" placeholder="Nombre o teléfono"></label><label>Paciente<select id="booking-patient" name="patient" required>${bookingPatientOptions(draft)}</select></label><p id="booking-search-note" class="field-help">El contacto ayuda a distinguir pacientes con el mismo nombre.</p>${button(icon('plus')+' Registrar paciente','inline-patient','text')}${patient?.needsResponsible?`<label>Responsable de contacto<select name="responsibleContactId" id="booking-responsible" required><option value="">Elige un responsable</option>${responsibleContacts.filter(c=>c.active!==false&&c.phoneCode&&c.phoneNumber&&patient.responsibleIds.includes(c.id)).map(c=>`<option value="${c.id}" ${draft.responsibleContactId===c.id?'selected':''}>${esc(c.name)} · ${esc(c.relationship)} · ${esc(phoneText(c))}</option>`).join('')}</select><small>Esta ficha requiere un responsable vigente con teléfono.</small></label>${button(icon('plus')+' Nuevo responsable','inline-responsible','text')}`:''}</div></fieldset>`;
  const professional=original?'':`${showProfessional()?`<label>Profesional<select name="professionalId" id="booking-professional">${professionals.slice(0,state.professionalCount).map(p=>`<option value="${p.id}" ${draft.professionalId===p.id?'selected':''}>${esc(p.name)}</option>`).join('')}</select></label>`:'<input type="hidden" name="professionalId" value="elena">'}<label>Servicio<select name="service" id="booking-service">${options(offeredNames(),draft.service)}</select></label>`;
  const coverageText=covered(draft.date)?'Disponibilidad local: 19–30 de octubre. Consultar no reserva el intervalo.':'Esta fecha no tiene cobertura local. Elige una fecha disponible antes de guardar.';
  const body=`${!canEditAppointments()?notice('Solo lectura','No tienes permiso para modificar las citas de esta consulta.'):''}<form id="appointment-form" data-id="${id}" ${activeDraft?'data-dirty="true"':''}><div class="form-layout"><section><fieldset class="booking-fields" ${canEditAppointments()?'':'disabled'}>${identity}<fieldset class="form-section"><legend>${original?'Nuevo horario':'Servicio y horario'}</legend><div class="fields">${professional}<label>Fecha<input type="date" name="date" id="booking-date" value="${esc(draft.date)}" required></label><div class="two-fields"><label>Desde<input type="time" name="start" id="booking-start" value="${esc(draft.start)}" required></label><label>Hasta<input type="time" name="end" id="booking-end" value="${esc(draft.end)}" ${original?'readonly':''} required></label></div>${original?`<p class="field-help">Se conservan los ${duration} minutos de la cita original.</p>`:`<p class="field-help">Duración de referencia del servicio: ${referenceDuration(draft.service)} minutos. Revisa el intervalo antes de guardar.</p>`}<p class="field-help">Hora de Caracas · ${coverageText}</p>${candidate?button('Usar '+candidate.start+' – '+candidate.end,'choose-slot','quiet',`data-start="${candidate.start}" data-end="${candidate.end}"`):'<p class="field-help">No hay una opción disponible para ese intervalo en los datos conocidos.</p>'}</div></fieldset></fieldset></section><aside class="summary" id="appointment-summary" aria-label="Resumen de la cita">${editorSummary(draft,original)}</aside></div><div id="form-error" class="error" role="alert"></div><div class="form-actions"><button class="primary" type="submit" ${canEditAppointments()?'':'disabled'}>${original?'Guardar nuevo horario':'Guardar cita'}</button>${link('Descartar',original?'cita/'+id:state.agendaView,'button')}</div></form>`;
  return shell(head(original?'Reprogramar cita':'Nueva cita',original?'Cambia el horario conservando los datos de la cita.':'Busca al paciente y revisa el intervalo antes de guardar.')+body,'Agenda','agenda',original?'Reprogramar':'Nueva cita');
}
function editorSummary(draft,original=null){
  const patient=state.patients.find(p=>p.id===draft.patient);
  const responsible=responsibleContacts.find(c=>c.id===draft.responsibleContactId);
  return `<h3>${original?'Comparar horarios':'Resumen de la cita'}</h3><div class="summary-title">${patient?esc(patient.name):'Selecciona un paciente'}</div><p>${esc(original?.service||draft.service)}</p><p>${esc(original?.professionalSnapshot||professionalName(draft.professionalId))}</p>${responsible?`<p>Responsable: ${esc(responsible.name)}</p>`:''}${original?`<div class="summary-original"><strong>Horario actual</strong><p>${titleCase(fullDate(original.date))}</p><p>${original.start} – ${original.end}</p></div>`:''}<div class="summary-date"><strong>${original?'Nuevo horario':'Horario elegido'}</strong><p>${icon('calendar')} ${titleCase(fullDate(draft.date||TODAY))}</p><p>${icon('clock')} ${esc(draft.start)} – ${esc(draft.end)}</p><p>${icon('pin')} ${esc(original?.location||accountLocation())}</p></div><div class="summary-footer">${original?'La confirmación anterior dejará de ser válida cuando se acepte el cambio.':'Guardar no equivale a enviar un recordatorio.'}</div>`;
}
function registerInlinePatient(){
  activeDraft=formValues(document.getElementById('appointment-form'));
  const draft=state.inlinePatientDraft||{};
  dialog('Registrar paciente',`<form id="inline-patient-form"><p>Registro con documento propio y teléfono de contacto.</p><div class="fields"><label>Nombre y apellidos<input name="name" value="${esc(draft.name||'')}" required></label><div class="two-fields"><label>País emisor<select name="country"><option value="VE">Venezuela</option></select></label><label>Tipo de documento<select name="documentType"><option value="national_id">Documento nacional</option><option value="passport" ${draft.documentType==='passport'?'selected':''}>Pasaporte</option></select></label></div><label>Número de documento<input name="documentNumber" value="${esc(draft.documentNumber||'')}" required></label><div class="two-fields"><label>Código de teléfono<input name="phoneCode" value="${esc(draft.phoneCode||'+58')}" required></label><label>Número de teléfono<input name="phone" type="tel" value="${esc(draft.phone||'')}" required></label></div></div><div id="inline-patient-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Registrar y volver a la cita</button></div></form>`);
}

function patients(){
  const records=state.patients.filter(p=>(p.name+' '+phoneText(p)).toLocaleLowerCase('es').includes(state.query.toLocaleLowerCase('es')));
  const row=p=>{
    const next=state.appointments.find(a=>a.patient===p.id&&a.date>=TODAY&&a.status!=='Cancelada');
    const last=state.appointments.filter(a=>a.patient===p.id).sort((a,b)=>(b.date+b.start).localeCompare(a.date+a.start))[0];
    return `<tr><td><a href="#/paciente/${p.id}" class="link-row">${avatar(p.name,p.id)}<span class="name">${esc(p.name)}</span></a></td><td class="muted mono">${esc(phoneText(p)||((p.responsibleIds||[]).length?'Por responsable':'Sin teléfono'))}</td><td class="contact-col muted">${next?`${fmt(next.date)}, ${next.start}`:'Sin cita'}</td><td class="activity-col muted">${last?`${fmt(last.date)} · ${esc(last.status.toLowerCase())}`:'Sin citas conocidas'}</td><td>${completionBadge(p)}</td><td>${patientStatusBadge(p)}</td></tr>`;
  };
  const listing=records.length?`<table class="data-table"><thead><tr><th><span class="column-label">${icon('people')} Nombre</span></th><th><span class="column-label">${icon('phone')} Contacto</span></th><th class="contact-col"><span class="column-label">${icon('calendar')} Próxima cita</span></th><th class="activity-col">Actividad</th><th>Ficha</th><th>Estado</th></tr></thead><tbody>${records.map(row).join('')}</tbody></table>`:state.patients.length?empty('No encontramos pacientes en este equipo','La búsqueda solo recorre los datos disponibles localmente. Prueba con otro nombre o número.',button('Limpiar búsqueda','clear-search'),'people'):empty('No hay pacientes en este equipo','Registra el primer paciente para empezar a organizar sus citas.',link('Registrar paciente','pacientes/nuevo','button'),'people');
  return shell(head('Pacientes','La información necesaria para organizar su próxima cita.',link(icon('plus')+' Registrar paciente','pacientes/nuevo','button primary'))+`<div class="toolbar"><div class="search">${icon('search')}<input type="search" id="patient-search" value="${esc(state.query)}" placeholder="Buscar por nombre o teléfono…" aria-label="Buscar pacientes"></div><span class="small subtle">${records.length} de ${state.patients.length} pacientes</span></div><div class="table-wrap">${listing}</div>`,'Pacientes','pacientes');
}
function patientRecord(id){
  const p=getPatient(id),records=state.appointments.filter(a=>a.patient===p.id).sort((a,b)=>(a.date+a.start).localeCompare(b.date+b.start));
  const missing=patientMissing(p);
  state.patientHistory=state.patientHistoryFor===id?state.patientHistory||5:5;state.patientHistoryFor=id;
  const facts=[['Documento',`${identifierText(p)}${p.identifier?' · '+verificationText(p.identifier.status):''}`]];
  if(p.birthDate)facts.push(['Nacimiento',longDate(p.birthDate)]);
  facts.push(['Teléfono',phoneText(p)||'No registrado'],['Correo',p.email||'No registrado'],['Contacto preferido',p.contact||'No definida']);
  const upcoming=records.filter(a=>a.date>=TODAY),history=records.filter(a=>a.date<TODAY);
  const responsables=(p.responsibleIds||[]).map(rid=>responsibleContacts.find(c=>c.id===rid)).filter(Boolean);
  const responsableRow=c=>`<div class="list-item"><div class="responsible-person">${avatar(c.name,c.id)}<div><strong>${esc(c.name)}</strong><p>${esc(c.relationship)} · ${esc(c.role||'Responsable administrativo')}</p><small>${esc(responsibleVigencia(c))} · Representación ${esc((representationLabel[c.representation]||'Declarada').toLowerCase())} · Teléfono ${esc(phoneText(c)||'incompleto')}</small><small>${esc(responsibleActivity(c))}</small></div></div><div class="item-actions">${button('Editar',`edit-responsible:${c.id}:${p.id}`,'quiet')}${c.active!==false?button('Inactivar vínculo',`deactivate-responsible:${c.id}:${p.id}`,'quiet'):button('Reactivar vínculo',`reactivate-responsible:${c.id}:${p.id}`,'quiet')}</div></div>`;
  return shell(`${p.completion!=='complete'?notice('Ficha incompleta',`Faltan: ${missing.join(' · ')||'requisitos administrativos'}. Guardar un borrador conserva estos datos; una ficha completa permite citar al paciente.`,button(icon('edit')+' Editar información',`edit-patient:${p.id}`,'button')):''}${p.active===false?notice('Ficha inactiva','Las citas existentes no se cancelan. La ficha no aparece al crear citas hasta que se reactive.'):''}<div class="record-header">${avatar(p.name,p.id)}<div><h1>${esc(p.name)}</h1><p>Paciente desde octubre de 2026</p><div class="head-actions" style="margin-top:10px">${completionBadge(p)}${patientStatusBadge(p)}</div></div></div><div class="record-grid"><aside class="record-aside"><h3>Información del paciente</h3><div class="facts">${facts.map(([label,value])=>`<div class="fact"><span>${label}</span><span>${esc(value)}</span></div>`).join('')}</div>${phonePairInvalid(p)?`<p class="field-help">El teléfono guardado está incompleto: falta ${p.phoneCode?'el número':'el código'}.</p>`:''}${button(icon('edit')+' Editar información',`edit-patient:${p.id}`,'quiet')}${button(p.active===false?'Reactivar ficha':'Inactivar ficha',`toggle-patient:${p.id}`,'quiet')}</aside><section><div class="page-head"><h2>Próximos encuentros</h2>${p.active!==false&&canEditAppointments()?link(icon('plus')+' Nueva cita',`cita/nueva?paciente=${p.id}`,'button primary'):''}</div>${upcoming.length?`<div class="table-wrap">${appointmentRows(upcoming)}</div>`:empty('Sin citas próximas','No hay encuentros futuros para este paciente en los datos disponibles.',p.active!==false&&canEditAppointments()?link('Nueva cita',`cita/nueva?paciente=${p.id}`,'button'):'')}<div class="section-title"><h2>Historial</h2><span class="small subtle">${Math.min(state.patientHistory,history.length)} de ${history.length} citas</span></div>${history.length?`<div class="table-wrap">${appointmentRows(history.slice(0,state.patientHistory))}</div>${history.length>state.patientHistory?`<div class="detail-actions">${button('Cargar más','more-patient-history')}</div>`:''}`:empty('Sin historial en este equipo','Las citas anteriores no están disponibles en los datos descargados.','')}<div class="section-line"><h3>Responsables</h3>${responsables.length?responsables.map(responsableRow).join(''):'<p class="muted small">Sin responsables vinculados.</p>'}<div class="detail-actions space-top">${button(icon('plus')+' Añadir responsable',`add-responsible:${p.id}`,'button')}</div><p class="small subtle space-top">Parentesco, vigencia, representación y actividad son datos separados. El documento de un adulto no identifica al menor.</p></div><div class="section-line"><h3>Preferencias de comunicación</h3><p class="muted small">${p.contact==='WhatsApp'?'WhatsApp para avisos administrativos de sus citas.':'Sin preferencia de contacto definida.'}</p></div></section></div>`,'Pacientes','pacientes',p.name);
}
function patientIdentityFields(p={}){
  const basis=p.needsResponsible?'minor_without_document':(p.identificationBasis||'own_document');
  const responsables=responsibleContacts.filter(c=>c.active!==false&&c.phoneCode&&c.phoneNumber);
  return `<fieldset class="form-section"><legend>Identificación</legend><div class="fields"><label>Nombre y apellidos<input name="name" autocomplete="off" value="${esc(p.name||'')}" required placeholder="Nombre del paciente"></label><div class="two-fields"><label class="radio-line"><input type="radio" name="identificationBasis" value="own_document" ${basis==='own_document'?'checked':''}>Tiene documento propio</label><label class="radio-line"><input type="radio" name="identificationBasis" value="minor_without_document" ${basis==='minor_without_document'?'checked':''}>Menor sin documento</label></div><div class="fields" data-basis="own_document" ${basis==='own_document'?'':'hidden'}><div class="two-fields"><label>País emisor<select name="identifierCountry"><option value="VE" ${p.identifier?.country==='VE'?'selected':''}>Venezuela</option></select></label><label>Tipo de documento<select name="identifierType"><option value="national_id" ${p.identifier?.type!=='passport'?'selected':''}>Documento nacional</option><option value="passport" ${p.identifier?.type==='passport'?'selected':''}>Pasaporte</option></select></label></div><label>Número de documento<input name="identifierNumber" value="${esc(p.identifier?.number||'')}" placeholder="Con prefijos y ceros, si los tiene"></label><p class="field-help">Conservamos prefijos, ceros y puntuación significativa; no lo tratamos como número. En este ejemplo el documento queda declarado, no verificado.</p></div><div class="fields" data-basis="minor_without_document" ${basis==='minor_without_document'?'':'hidden'}><label>Fecha de nacimiento<input type="date" name="birthDate" value="${esc(p.birthDate||'')}"></label><p class="field-help">Al citar se comprueba que sea menor de 18 en la fecha local de la cita. El documento de un adulto no identifica al menor.</p></div><label>Responsable de contacto <span class="small subtle">${basis==='minor_without_document'?'Necesario':'Opcional si no hay teléfono propio'}</span><select name="responsibleId"><option value="">Sin responsable</option>${responsables.map(c=>`<option value="${c.id}" ${p.responsibleIds?.includes(c.id)?'selected':''}>${esc(c.name)} · ${esc(c.relationship)} · ${esc(phoneText(c))}</option>`).join('')}</select><small>El responsable debe estar vigente y tener teléfono.</small></label></div></fieldset>`;
}
function patientContactFields(p={}){
  return `<fieldset class="form-section"><legend>Contacto</legend><div class="fields"><div class="two-fields"><label>Código de teléfono<input name="phoneCode" value="${esc(p.phoneCode||'')}" placeholder="+58"></label><label>Número de teléfono<input name="phoneNumber" type="tel" value="${esc(p.phoneNumber||'')}" placeholder="412 000 0000"></label></div><p class="field-help">Código y número forman un teléfono: complétalos juntos o déjalos vacíos. Un teléfono no acredita identidad.</p>${phonePairInvalid(p)?`<p class="field-help">El teléfono guardado está incompleto: falta ${p.phoneCode?'el número':'el código'}.</p>`:''}<label>Correo electrónico <span class="small subtle">Opcional</span><input name="email" type="email" value="${esc(p.email||'')}" placeholder="nombre@example.test"></label><label class="check-line"><input name="contact" type="checkbox" ${p.contact&&p.contact!=='WhatsApp'?'':'checked'}>Contacto por WhatsApp para avisos de citas</label></div></fieldset>`;
}
function patientForm(){return shell(head('Registrar paciente','Solo datos administrativos para gestionar sus citas. Puedes guardar un borrador y completarla después.',link('Volver a Pacientes','pacientes','button quiet'))+`<form id="patient-form" style="max-width:640px">${patientIdentityFields()}${patientContactFields()}<div id="form-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Registrar paciente</button>${button('Guardar borrador','save-patient-draft','quiet')}${link('Descartar','pacientes','button')}</div></form>`,'Pacientes','pacientes','Registrar');}
function responsibleFields(c={}){
  return `<div class="fields"><label>Nombre y apellidos<input name="name" value="${esc(c.name||'')}" required></label><div class="two-fields"><label>Parentesco<input name="relationship" value="${esc(c.relationship||'')}" placeholder="Madre, padre, tutor legal…" required></label><label>Función<select name="role">${options(['Contacto y decisiones administrativas','Solo contacto'],c.role||'Contacto y decisiones administrativas')}</select></label></div><div class="two-fields"><label>Código de teléfono<input name="phoneCode" value="${esc(c.phoneCode||'')}" placeholder="+58"></label><label>Número de teléfono<input name="phoneNumber" type="tel" value="${esc(c.phoneNumber||'')}"></label></div><p class="field-help">Código y número forman un teléfono: complétalos juntos.</p><label>Constancia de representación<select name="representation">${[['declared','Declarada'],['documented','Documentada'],['none','Sin constancia']].map(([v,l])=>`<option value="${v}" ${(c.representation||'declared')===v?'selected':''}>${l}</option>`).join('')}</select><small>Es un dato separado del parentesco y de la vigencia.</small></label></div>`;
}
function editResponsible(contactId,patientId){const c=responsibleContacts.find(c=>c.id===contactId);dialog('Editar responsable',`<form id="responsible-form" data-contact="${contactId}" data-patient="${patientId}">${responsibleFields(c)}<div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Guardar cambios</button></div></form>`);}
function addResponsible(patientId){dialog('Añadir responsable',`<form id="responsible-form" data-patient="${patientId}">${responsibleFields()}<div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Añadir responsable</button></div></form>`);}
const intervalRows = intervals => intervals.map(i=>`<div class="interval-row"><div class="two-fields"><label>Inicio<input type="time" class="rule-start" value="${i.start}" required></label><label>Fin<input type="time" class="rule-end" value="${i.end}" required></label></div><p class="interval-duration">${intervalDuration(i.start,i.end)}</p></div>`).join('');
function intervalDuration(start,end){const duration=minutes(end)-minutes(start);return start&&end&&duration>0?`${Math.floor(duration/60)?Math.floor(duration/60)+' h ':''}${duration%60?duration%60+' min ':''}de atención`:'Elige una hora de fin posterior al inicio.';}
function ruleForm(r={}){
  return `<form id="rule-form" data-rule="${r.id||''}"><div class="fields"><fieldset class="schedule-days"><legend>1. Días de atención</legend><p class="field-help">Este horario se repite en los días que marques.</p><div>${[1,2,3,4,5,6].map(d=>`<label><input type="checkbox" name="day" value="${d}" ${(r.days||[1,2,3,4,5]).includes(d)?'checked':''}><span>${dayNames[d].slice(0,3)}</span></label>`).join('')}</div></fieldset><section class="schedule-form-section"><h3>2. Vigencia</h3><div class="two-fields"><label>Desde el<input type="date" name="validFrom" value="${r.validFrom||coverage.from}" required></label><label>Hasta el<input type="date" name="validTo" value="${r.validTo||coverage.to}" required></label></div></section><section class="schedule-form-section"><h3>3. Tramos de atención</h3><p class="field-help">Añade un tramo por cada período de atención. Deja los descansos entre ellos.</p><div id="interval-rows">${intervalRows(r.intervals||[{start:'09:00',end:'12:00'}])}</div>${button(icon('plus')+' Añadir tramo','add-interval-row','quiet')}</section></div><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">${r.id?'Guardar cambios':'Crear regla'}</button></div></form>`;
}
function exceptionForm(ex={}){
  return `<form id="exception-form" data-exception="${ex.id||''}"><div class="fields"><label>Fecha<input type="date" name="date" value="${ex.date||''}"></label><label>Tipo<select name="exType">${[['closed','Cierre del día'],['replacement','Reemplazo de horario']].map(([v,l])=>`<option value="${v}" ${(ex.type||'closed')===v?'selected':''}>${l}</option>`).join('')}</select></label><div data-ex-intervals ${(!ex.type||ex.type==='closed')?'hidden':''}><div id="ex-interval-rows">${intervalRows(ex.intervals&&ex.intervals.length?ex.intervals:[{start:'09:00',end:'13:00'}])}</div>${button(icon('plus')+' Añadir tramo','add-ex-interval-row','quiet')}</div><label>Motivo <span class="small subtle">Opcional</span><input name="label" value="${esc(ex.label||'')}" placeholder="Por ejemplo, mantenimiento"></label><p class="field-help">El cierre deja el día sin atención; el reemplazo sustituye el horario habitual por los tramos indicados.</p></div><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">${ex.id?'Guardar cambios':'Crear excepción'}</button></div></form>`;
}
function blockForm(b={}){
  return `<form id="block-form" data-block="${b.id||''}"><div class="fields"><label>Fecha<input type="date" name="date" value="${b.date||state.date}"></label><div class="two-fields"><label>Desde<input type="time" name="start" value="${b.start||'12:00'}"></label><label>Hasta<input type="time" name="end" value="${b.end||'13:00'}"></label></div><label>Alcance<select name="scope">${[['account','Esta consulta'],['professional','Toda la actividad del profesional']].map(([v,l])=>`<option value="${v}" ${(b.global?'professional':'account')===v?'selected':''}>${l}</option>`).join('')}</select></label><label>Motivo<input name="label" value="${esc(b.label||'')}" placeholder="Por ejemplo, gestiones personales"></label><p class="field-help">Los asistentes no ven el motivo de los bloqueos de toda la actividad.</p></div><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">${b.id?'Guardar cambios':'Crear bloqueo'}</button></div></form>`;
}

function requests(){
  const pending=state.requests.filter(r=>r.status==='pending');
  const row=r=>{const a=getAppointment(r.appointmentId),p=getPatient(a.patient);return `<div class="list-item"><div><a href="#/solicitud/${r.id}" class="link-row">${avatar(p.name,p.id)}<strong>${esc(p.name)}</strong>${requestBadge(r)}</a><p>Solicita cambiar su cita del ${fmt(a.date)}, ${a.start}.</p><small>Recibida ${ageText(r.received)}</small></div>${link('Revisar solicitud','solicitud/'+r.id,'button')}</div>`;};
  return shell(head('Solicitudes','Cambios que los pacientes han pedido desde su enlace de cita.')+(state.requests.length?state.requests.map(row).join(''):empty('No hay solicitudes','Cuando un paciente pida un cambio desde su enlace de cita, aparecerá aquí.',link('Volver a Agenda','agenda','button'),'inbox'))+(pending.length?`<p class="small subtle space-top">${pending.length} solicitud(es) esperan tu revisión.</p>`:'<p class="small subtle space-top">Nada pendiente: el resto quedó resuelto, desactualizado o en conciliación.</p>'),'Solicitudes','solicitudes');
}
function requestDetail(id){
  const r=state.requests.find(r=>r.id===id);
  if(!r)return shell(head('Solicitud no disponible')+empty('No encontramos esta solicitud','Vuelve a la lista para ver las solicitudes del ejemplo.',link('Volver a Solicitudes','solicitudes','button')),'Solicitudes','solicitudes');
  const a=getAppointment(r.appointmentId),p=getPatient(a.patient);
  const actions=r.status==='pending'?`<div class="section-line"><h3>Próximo paso</h3><p class="muted small">Busca otro horario y guarda el cambio; la solicitud se actualizará al registrarlo. La cita conserva su horario mientras tanto.</p><div class="detail-actions space-top">${link(icon('calendar')+' Elegir nuevo horario',`cita/${a.id}/reprogramar?solicitud=${r.id}`,'button primary')}</div></div>`:`<div class="section-line"><h3>${r.status==='partial'?'Conciliación':'Estado'}</h3><p class="muted small">${esc(requestStatusText(r))}</p><div class="detail-actions space-top">${link('Ver cita','cita/'+a.id,'button')}</div></div>`;
  return shell(head('Solicitud de cambio','El paciente pide otra fecha. Su cita mantiene el horario actual.',link('Volver a Solicitudes','solicitudes','button quiet'))+`<div class="form-layout"><section><div class="patient-heading">${avatar(p.name,p.id)}<div><h2>${esc(p.name)}</h2><p class="small subtle">${esc(phoneText(p)||'No registrado')}</p></div></div><div class="section-line"><h3>Lo que pide el paciente</h3><p class="muted" style="margin-top:12px">«${esc(r.message)}»</p><p class="small subtle space-top">Recibida ${ageText(r.received)}</p></div><div class="section-line"><h3>Estado de la solicitud</h3>${requestBadge(r)}<p class="muted small" style="margin-top:10px">${esc(requestStatusText(r))}</p></div>${actions}</section><aside class="summary"><h3>Cita actual</h3><div class="summary-title">${fmt(a.date)}</div><p>${a.start} – ${a.end}</p><p>${esc(a.service)}</p><p>${esc(a.professionalSnapshot||professionalName(a.professionalId))}</p><p>Consultorio 204</p><div class="summary-footer">La solicitud por sí sola no reserva ni mueve esta cita.</div></aside></div>`,'Solicitudes','solicitudes','Revisar');
}

function waitlist(){
  const waiting=state.waitlist.length?`<div class="table-wrap"><table class="data-table"><thead><tr><th>Paciente</th><th>Servicio</th><th>Preferencia</th><th>En espera desde</th><th>Estado</th><th class="row-actions"><span class="sr-only">Acciones</span></th></tr></thead><tbody>${state.waitlist.map(w=>{const p=getPatient(w.patient);return `<tr><td><a class="link-row" href="#/paciente/${p.id}">${avatar(p.name,p.id)}<span class="name">${esc(p.name)}</span></a></td><td class="muted">${esc(w.service)}</td><td class="muted">${esc(w.preference)}</td><td class="muted">${fmt(w.since)}</td><td>${w.assigned?'<span class="badge success">Asignada</span>':'<span class="badge">En espera</span>'}</td><td class="row-actions">${w.assigned?'':button(icon('edit'),`edit-waitlist:${w.id}`,'quiet icon',`aria-label="Editar entrada de ${esc(p.name)}"`)+button('Ofrecer horario',`offer-candidates:${w.id}`,'quiet')}</td></tr>`;}).join('')}</tbody></table></div><p class="small subtle space-top">El orden de esta lista no fija una prioridad de asignación; la política de prioridad sigue pendiente.</p>`:empty('Lista de espera vacía','Añade pacientes cuando no quede un horario que les sirva.',button(icon('plus')+' Añadir paciente','add-waitlist','button'));
  return shell(head('Lista de espera','Encuentra pacientes compatibles cuando quede un horario libre.',button(icon('plus')+' Añadir paciente','add-waitlist','primary'))+waiting,'Lista de espera','espera');
}

function communications(){
  const statuses=['Programado','Entregado','Respondido','Revisar teléfono','Resultado desconocido'];
  const rows=state.messages.filter(m=>!state.commFilter||m.status===state.commFilter);
  const filterNote=state.commFilter?`<p class="small space-top">Filtrado por «${esc(state.commFilter)}» ${button('Limpiar filtro','clear-comm-filter','quiet')}</p>`:'';
  const body=rows.length?`<div class="table-wrap"><table class="data-table"><thead><tr><th>Paciente</th><th>Comunicación</th><th>Cita relacionada</th><th>Envío</th><th>Estado</th><th class="row-actions"><span class="sr-only">Acciones</span></th></tr></thead><tbody>${rows.map(m=>{const p=getPatient(m.patient),a=getAppointment(m.appointmentId);return `<tr><td><a href="#/paciente/${p.id}" class="link-row">${avatar(p.name,p.id)}<span class="name">${esc(p.name)}</span></a></td><td class="muted">${esc(m.type)} · ${esc(m.channel)}</td><td class="muted">${a?`<a class="link-row" href="#/cita/${a.id}">${fmt(a.date)}, ${a.start}</a>`:'—'}</td><td class="muted mono">${fmt(m.date)} · ${m.time}</td><td>${badge(m.status)}</td><td class="row-actions">${button('Ver detalle',`message:${m.id}`,'quiet')}</td></tr>`;}).join('')}</tbody></table></div>`:empty('Sin comunicaciones con este filtro','Prueba con otro estado o limpia el filtro para ver todos los mensajes del ejemplo.',button('Limpiar filtro','clear-comm-filter','button'));
  return shell(head('Comunicaciones','Recordatorios y respuestas relacionados con las citas. No hay mensajería libre.',link(icon('settings')+' Política de avisos','configuracion/comunicaciones','button'))+`<div class="toolbar"><span class="small muted">Mensajes recientes · WhatsApp</span>${button('Filtros '+icon('down'),'comm-filters','button')}<span class="small subtle">Datos de ejemplo</span></div>`+body+filterNote+`<p class="small subtle space-top">Programado, enviado, entregado, respondido o fallido: ningún estado de comunicación cambia por sí solo el estado de la cita.</p>`,'Comunicaciones','comunicaciones');
}
const metricPeriods=[
  {id:'week-current',label:'Semana en curso (19 – 25 oct)',updated:'21 de octubre, 23:40',open:true,registered:24,attended:16,absent:2,recovered:1,weekly:[{label:'19 – 21 oct',value:24}],confirmations:15,cancellations:3},
  {id:'week-last',label:'Semana pasada (12 – 18 oct)',updated:'18 de octubre, 23:40',registered:22,attended:18,absent:2,recovered:1,weekly:[{label:'12 – 18 oct',value:22}],confirmations:17,cancellations:3},
  {id:'two-weeks',label:'Últimas dos semanas (8 – 21 oct)',updated:'21 de octubre, 23:40',open:true,registered:46,attended:34,absent:4,recovered:2,weekly:[{label:'8 – 11 oct',value:20},{label:'12 – 15 oct',value:12},{label:'16 – 18 oct',value:10},{label:'19 – 21 oct',value:4}],confirmations:34,cancellations:6},
  {id:'month',label:'Octubre (1 – 31 oct)',updated:'21 de octubre, 23:40',open:true,registered:64,attended:45,absent:5,recovered:4,weekly:[{label:'1 – 7 oct',value:18},{label:'8 – 14 oct',value:22},{label:'15 – 21 oct',value:24}],confirmations:48,cancellations:8},
  {id:'next-week',label:'Semana del 26 oct – 1 nov',updated:'',empty:true}
];
function metrics(){
  const current=metricPeriods.find(p=>p.id===state.metricPeriod)||metricPeriods[3];
  const toolbar=`<div class="toolbar">${button(current.label+' '+icon('down'),'metric-filters','button')}<span class="small subtle">Informe ficticio · Actualizado el ${current.updated||'—'} · Hora de Caracas</span></div>`;
  if(state.scenario==='unavailable')return shell(head('Indicadores','Citas y resultados administrativos de la consulta.')+toolbar+empty('No pudimos leer el informe','El servicio de indicadores no está disponible en este ejemplo. La agenda y el resto de las pantallas siguen utilizables con los datos locales.',button('Volver a cargar','metrics-retry','button'),'info'),'Indicadores','indicadores');
  if(current.empty)return shell(head('Indicadores','Citas y resultados administrativos de la consulta.')+toolbar+empty('Sin datos en este periodo','Todavía no hay citas registradas entre el 26 de octubre y el 1 de noviembre. El informe aparecerá cuando existan datos.',button('Ver semana en curso','metric-period:week-current','button'),'chart'),'Indicadores','indicadores');
  const finalized=current.attended+current.absent;
  const attPct=Math.round(current.attended/finalized*100),absPct=Math.round(current.absent/finalized*100),confPct=Math.round(current.confirmations/current.registered*100),cancelPct=Math.round(current.cancellations/current.registered*100);
  const max=Math.max(...current.weekly.map(w=>w.value));
  const openNote=current.open?`<p class="small subtle space-top">Periodo en curso: los datos llegan hasta el 21 de octubre y las cifras pueden crecer al cerrarse.</p>`:'';
  return shell(head('Indicadores','Citas y resultados administrativos de la consulta.')+toolbar+openNote+`<div class="metric-row"><div class="metric"><p>Citas registradas</p><div class="value mono">${current.registered}</div><small>citas · ${esc(current.label)}</small></div><div class="metric"><p>Asistencia</p><div class="value mono">${attPct} %</div><small>${current.attended} atendidas de ${finalized} finalizadas</small></div><div class="metric"><p>Ausencias sin aviso</p><div class="value mono">${current.absent}</div><small>${absPct} % de las ${finalized} finalizadas</small></div><div class="metric"><p>Horarios recuperados</p><div class="value mono">${current.recovered}</div><small>citas · desde la lista de espera</small></div></div><div class="section-title"><h2>Citas registradas por semana</h2><span class="small subtle">${current.registered} citas en total</span></div><div class="chart" role="img" aria-label="Citas por tramo del periodo: ${current.weekly.map(w=>`${w.value} (${w.label})`).join(', ')}">${current.weekly.map(w=>`<div class="bar-group"><span>${w.value}</span><div class="bar" style="height:${Math.round(w.value/max*80)}%"></div><span>${w.label}</span></div>`).join('')}</div><p class="small subtle">Escala: cada barra se normaliza al valor máximo del periodo (${max}).</p><div class="table-wrap space-bottom space-top"><table class="data-table"><thead><tr><th>Tramo del periodo</th><th>Citas registradas</th></tr></thead><tbody>${current.weekly.map(w=>`<tr><td>${w.label}</td><td>${w.value}</td></tr>`).join('')}<tr><td><strong>Total</strong></td><td><strong>${current.registered}</strong></td></tr></tbody></table></div><div class="table-wrap space-bottom"><table class="data-table"><thead><tr><th>Seguimiento del periodo</th><th>Total</th><th>Sobre las citas registradas</th></tr></thead><tbody><tr><td>Confirmaciones recibidas</td><td>${current.confirmations}</td><td>${current.confirmations} de ${current.registered} · ${confPct} %</td></tr><tr><td>Cancelaciones</td><td>${current.cancellations}</td><td>${current.cancellations} de ${current.registered} · ${cancelPct} %</td></tr></tbody></table></div><details><summary class="small muted">Cómo se calculan estos indicadores</summary><div class="space-top"><table class="data-table"><thead><tr><th>Indicador</th><th>Unidad</th><th>Cálculo del ejemplo</th></tr></thead><tbody><tr><td>Citas registradas</td><td>citas</td><td>Suma de citas del periodo, sin contar cancelaciones dos veces</td></tr><tr><td>Asistencia</td><td>%</td><td>${current.attended} atendidas / (${current.attended} atendidas + ${current.absent} ausencias)</td></tr><tr><td>Ausencias</td><td>%</td><td>${current.absent} ausencias / ${finalized} citas finalizadas</td></tr><tr><td>Confirmaciones y cancelaciones</td><td>citas y %</td><td>Sobre las ${current.registered} citas registradas del periodo</td></tr></tbody></table></div></details>`,'Indicadores','indicadores');
}

function reviewItems(){
  const items=[];
  if(state.scenario==='conflict'){const a=getAppointment('a3'),p=getPatient(a.patient);items.push({id:'conflict',title:'Conflicto de horario',text:`${p.name} · Cita del ${fmt(a.date)}, ${a.start} – ${a.end}. Otro equipo reservó el intervalo; tu propuesta se conserva.`,action:link('Revisar horario','conflicto/a3','button')});}
  if(!state.reviewDismissed.includes('edit-rejected'))items.push({id:'edit-rejected',title:'Edición no aplicada',text:'El cambio de horario semanal del 21 de octubre fue rechazado: cruzaba 2 citas. La agenda compartida no cambió y tus datos se conservaron.',action:link('Revisar horario','configuracion/horarios','button')+button('Marcar revisado','dismiss-review:edit-rejected','quiet')});
  state.requests.filter(r=>r.status==='partial').forEach(r=>{const a=getAppointment(r.appointmentId),p=getPatient(a.patient);items.push({id:'partial-'+r.id,title:'Conciliación pendiente',text:`${p.name} · el cambio fue aceptado, pero la solicitud sigue abierta: falta actualizarla.`,action:link('Ver solicitud','solicitud/'+r.id,'button')});});
  devices.filter(d=>d.status==='revoking').forEach(d=>items.push({id:'dev-'+d.id,title:'Revocación por confirmar',text:`${d.name} · falta confirmar la revocación del acceso con el servicio.`,action:link('Ver dispositivos','configuracion/dispositivos','button')}));
  teamMembers.filter(m=>!m.active).forEach(m=>items.push({id:'member-'+m.id,title:'Acceso retirado por aplicar',text:`${m.name} · los equipos desconectados recibirán el cambio al conectarse.`,action:link('Ver equipo','configuracion/equipo','button')}));
  return items;
}
function activity(){
  const items=reviewItems();
  const content=items.length?items.map(i=>`<div class="list-item"><div><strong>${esc(i.title)}</strong><p>${esc(i.text)}</p></div><div class="item-actions">${i.action}</div></div>`).join('')+`<p class="small subtle space-top">Solo aparecen los problemas que necesitan una decisión. Las recuperaciones automáticas y los resultados por comprobar no crean avisos aquí.</p>`:empty('No hay asuntos por revisar','Cuando algo necesite tu decisión, aparecerá aquí y junto al dato afectado.',link('Ver Agenda','agenda','button'),'check');
  return shell(head('Por revisar','Problemas que necesitan tu atención.',link('Volver a Agenda','agenda','button quiet'))+content,'Por revisar','revision');
}

function conflict(){return shell(head('Revisar un horario','Otro equipo reservó el intervalo que elegiste.',link('Volver a Agenda','agenda','button quiet'))+`<div class="patient-heading">${avatar(getPatient('p3').name,'p3')}<div><h2>${getPatient('p3').name}</h2><p class="small subtle">Consulta general</p></div></div><div class="compare"><div><h3>Tu propuesta</h3><p>22 de octubre · 10:00 – 10:30</p><p class="small muted space-top">La conservamos para que puedas elegir otra hora.</p></div><div><h3>Horario compartido</h3><p>Ese intervalo ya está ocupado.</p><p class="small muted space-top">No se ha confirmado la reserva de este paciente en ese horario.</p></div></div><div class="detail-actions">${link('Elegir otro horario','cita/a3/reprogramar','button primary')}${button('Descartar mi propuesta','discard-conflict')}</div>`,'Agenda','agenda','Revisar horario');}

const settingsItems=[['Cuenta','cuenta'],['Servicios','servicios'],['Horarios y bloqueos','horarios'],['Comunicaciones','comunicaciones'],['Usuarios y permisos','equipo'],['Dispositivos','dispositivos']];
function settings(section){
  let content='';
  if(section==='cuenta'){
    const account=state.account,place=account.place,editable=state.viewerRole==='owner';
    const snapshotExample=state.appointments.find(a=>a.location);
    content=`<form id="account-form"><fieldset class="settings-fields" ${editable?'':'disabled'}><section class="settings-section"><h2>Datos de la consulta</h2><p>Información compartida que aparece en las citas nuevas. Cambiarla requiere conexión y permiso de administración.</p><div class="fields"><label>Nombre de la consulta<input name="practice" value="${esc(account.practice)}"></label><div class="two-fields"><label>Profesional<input name="professional" value="${esc(account.professional)}"></label><label>Especialidad<input name="specialty" value="${esc(account.specialty)}"></label></div></div></section><section class="settings-section"><h2>Lugar de atención</h2><p>Las citas muestran el nombre del lugar y la sala; la dirección completa queda en la cuenta y termina en ciudad y estado.</p><div class="fields"><div class="two-fields"><label>Nombre del lugar<input name="placeName" value="${esc(place.name)}"></label><label>Sala o consultorio<input name="placeRoom" value="${esc(place.room)}"></label></div><label>Dirección<input name="placeStreet" value="${esc(place.street)}" placeholder="Calle, número, piso"></label><div class="two-fields"><label>Ciudad<input name="city" value="${esc(place.city)}"></label><label>Estado<input name="state" value="${esc(place.state)}"></label></div><p class="field-help">La dirección de lectura llega hasta ciudad y estado; el país no se muestra.</p></div></section><section class="settings-section"><h2>Zona horaria</h2><p>Aplica a los horarios compartidos de la consulta.</p><label>Zona horaria<input value="América / Caracas (UTC −04:00)" readonly></label><p class="field-help">Cambiar la zona exige revisar los horarios; se gestiona junto a ellos en «Horarios y bloqueos».</p></section></fieldset>${editable?'':'<p class="field-help">Solo la persona que administra la consulta puede cambiar estos datos.</p>'}<div id="form-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary" ${editable?'':'disabled'}>Guardar cambios</button></div><p class="small subtle">Las citas ya registradas conservan el lugar y los datos guardados en su momento; cambiar la cuenta no los reescribe.${snapshotExample?` Ejemplo: una cita guardada conserva «${esc(snapshotExample.location)}».`:''}</p></form><form id="local-form"><section class="settings-section"><h2>Preferencias de este equipo</h2><p>Se guardan en este equipo y no requieren conexión.</p><div class="fields"><label>Formato de hora<select name="hourFormat">${['24 horas','12 horas'].map(v=>`<option ${state.localPrefs?.hourFormat===v?'selected':''}>${v}</option>`).join('')}</select></label><label>Tema<input value="Claro" readonly></label><p class="field-help">El tema claro es el único definido por ahora.</p></div></section><div class="form-actions"><button type="submit" class="primary">Guardar preferencias locales</button></div></form>`;
  }
  if(section==='servicios'){
    const editable=state.viewerRole==='owner';
    const rows=serviceOffers.map(o=>{
      const c=serviceCatalog.find(s=>s.id===o.serviceId);
      if(!c)return'';
      const effective=o.override||c.baseDuration;
      const adjust=o.override?`ajuste de la oferta sobre una base de ${c.baseDuration} min`:`sin ajuste; usa la base de ${c.baseDuration} min`;
      return `<div class="list-item"><div><strong>${esc(c.name)}</strong><p>Duración de referencia: ${effective} min · ${adjust}</p><small>Catálogo del profesional: ${c.active!==false?'activo':'inactivo'} · Oferta de esta consulta: ${o.active?'activa':'inactiva'}</small></div><div class="item-actions">${o.active?button('Desactivar oferta',`toggle-offer:${o.id}`,'quiet'):button('Activar oferta',`toggle-offer:${o.id}`,'quiet')}${c.active!==false?button('Inactivar en catálogo',`toggle-catalog:${c.id}`,'quiet'):button('Activar en catálogo',`toggle-catalog:${c.id}`,'quiet')}${editable?button(icon('edit'),`edit-service:${c.id}`,'quiet icon',`aria-label="Editar ${esc(c.name)}"`):''}</div></div>`;
    }).join('');
    content=`<div class="page-head"><div><h2>Catálogo y oferta</h2><p>El catálogo pertenece al profesional; la oferta ajusta la duración de referencia y la visibilidad en esta consulta.</p></div>${editable?button(icon('plus')+' Añadir servicio','add-service'):''}</div>${rows}<p class="small subtle space-top">Las citas ya registradas conservan su servicio y su duración aunque cambies el catálogo o la oferta. Las versiones técnicas no se muestran aquí.</p>${editable?'':'<p class="small subtle">Solo la persona que administra la consulta puede cambiar el catálogo y la oferta.</p>'}`;
  }
  if(section==='horarios'){
    const editable=state.viewerRole==='owner';
    const ruleCard=r=>`<article class="schedule-rule"><div class="section-title"><h3>Días de atención</h3>${editable?button(icon('edit')+' Editar horario',`edit-rule:${r.id}`,'quiet'):''}</div><div class="schedule-weekdays">${r.days.map(d=>`<span>${dayNames[d]}</span>`).join('')}</div><p class="schedule-validity">${icon('calendar')} Del <strong>${fmt(r.validFrom)}</strong> al <strong>${fmt(r.validTo)}</strong></p><div class="schedule-tracks">${r.intervals.map(i=>`<div><span>${icon('clock')}<strong>${i.start} <span>–</span> ${i.end}</strong></span><span>${intervalDuration(i.start,i.end)}</span></div>`).join('')}</div></article>`;
    content=`${!state.reviewDismissed.includes('edit-rejected')?notice('Edición rechazada el 21 de octubre','El cambio cruzaba 2 citas; la agenda compartida no cambió. Revisa antes de reintentar.',link('Ver en Por revisar','actividad','button')):''}<section class="settings-section"><div class="page-head"><div><h2>Horario semanal</h2><p>Define qué días atiendes, durante qué fechas y en qué horas. Hora de Caracas.</p></div>${editable?button(icon('plus')+' Añadir regla','add-rule'):''}</div>${scheduleRules.map(ruleCard).join('')||'<p class="muted small">Sin reglas definidas.</p>'}</section><div class="section-title"><h2>Excepciones por fecha</h2>${editable?button(icon('plus')+' Añadir excepción','add-exception'):''}</div>${scheduleExceptions.map(ex=>`<div class="list-item"><div><strong>${titleCase(fullDate(ex.date))}</strong><p>${ex.type==='closed'?'Cierre del día':`Reemplazo: ${ex.intervals.map(i=>i.start+' – '+i.end).join(' · ')}`}${ex.label?' · '+esc(ex.label):''}</p></div><div class="item-actions">${editable?button('Editar',`edit-exception:${ex.id}`,'quiet'):''}</div></div>`).join('')||'<p class="muted small">Sin excepciones.</p>'}<div class="section-title"><h2>Bloqueos</h2>${editable?button(icon('plus')+' Añadir bloqueo','add-block'):''}</div>${scheduleBlocks.map(b=>`<div class="list-item"><div><strong>${b.global&&state.viewerRole==='assistant'?'Bloqueo de agenda':esc(b.label)}</strong><p>${fmt(b.date)} · ${b.start} – ${b.end}</p><small>Alcance: ${b.global?'Toda la actividad del profesional':'Esta consulta'}</small></div><div class="item-actions">${editable?button('Editar',`edit-block:${b.id}`,'quiet'):''}</div></div>`).join('')}<p class="small subtle space-top">Un cambio que deje citas vigentes fuera del horario o dentro de un bloqueo se rechaza y muestra las citas afectadas; las citas no se mueven ni se cancelan desde aquí.</p>`;
  }
  if(section==='comunicaciones')content=`<form id="settings-form"><section class="settings-section"><h2>Recordatorios</h2><p>Recordatorios automáticos antes de la cita. Los valores marcados son ejemplos de revisión: la política definitiva debe acordarse con la consulta.</p><div class="fields"><label class="check-line"><input type="checkbox" name="reminder" checked> Recordatorio previo a la cita</label><label>Anticipación <span class="small subtle">Ejemplo de revisión · sin política aprobada</span><select name="reminderAdvance"><option>24 horas antes</option><option>48 horas antes</option></select></label><label class="check-line"><input type="checkbox" name="includeAccess" checked> Incluir el enlace a la cita del paciente</label></div></section><section class="settings-section"><h2>Acciones del paciente</h2><p>El paciente puede confirmar asistencia, cancelar o pedir un cambio desde su enlace, según la política vigente.</p><p class="small muted">El plazo de cancelación está pendiente de acuerdo; esta muestra no lo fija.</p></section><section class="settings-section"><h2>Enlaces de cita</h2><p>El enlace identifica una cita concreta del paciente.</p><p class="small muted">La vigencia del enlace y su verificación adicional no están contratadas; se muestran como diseño, no como regla del producto.</p></section><div id="form-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Guardar configuración</button></div><p class="small subtle">Guardar cambia el ejemplo; no envía mensajes ni modifica citas existentes.</p></form>`;
  if(section==='equipo'){
    const owner=state.viewerRole==='owner';
    const personRow=m=>`<div class="list-item"><div class="link-row">${avatar(m.name,m.id)}<div><strong>${esc(m.name)}</strong><p>${esc(m.email)}</p><small>${m.active?'Acceso activo':'Acceso retirado'}</small></div></div><div class="item-actions">${badge(m.role)}${button('Revisar acceso',`person:${m.id}`,'quiet')}</div></div>`;
    const inviteRow=i=>`<div class="list-item"><div><strong>${esc(i.email)}</strong><p>Invitación de ${esc(i.role)} · Enviada el ${fmt(i.sent)}</p></div><div class="item-actions">${i.status==='pending'?badge('Pendiente')+(owner?button('Retirar invitación',`revoke-invite:${i.id}`,'quiet'):''):badge('Vencida')+(owner?button('Invitar de nuevo',`reinvite:${i.id}`,'quiet'):'')}</div></div>`;
    content=`<div class="page-head"><div><h2>Personas de la consulta</h2><p>Cada persona usa su propia cuenta. Los roles son los definidos; no hay roles personalizados.</p></div>${owner?button(icon('plus')+' Invitar asistente','invite'):''}</div>${teamMembers.map(personRow).join('')}<div class="section-title"><h2>Invitaciones</h2></div>${invitations.length?invitations.map(inviteRow).join(''):'<p class="muted small">Sin invitaciones pendientes.</p>'}<p class="small subtle space-top">Cambiar permisos requiere conexión y autorización. Esta muestra no envía correos ni crea accesos reales.${owner?'':' Solo la persona que administra la consulta puede invitar o retirar accesos.'}</p>`;
  }
  if(section==='dispositivos'){
    const statusBadge=d=>d.status==='current'?badge('Actual'):d.status==='revoking'?badge('Por confirmar'):d.status==='revoked'?badge('Revocado'):badge('Autorizado');
    const owner=state.viewerRole==='owner';
    const next=d=>d.status==='revoking'?(owner?button('Comprobar de nuevo',`device-recheck:${d.id}`,'quiet'):''):d.status==='revoked'?(owner?button('Autorizar de nuevo',`device-reactivate:${d.id}`,'quiet'):''):'';
    content=`<h2>Equipos autorizados</h2><p class="muted small" style="margin-top:8px">Revisa desde qué equipos se accede a la consulta. Los secretos de sesión no se muestran.</p>${devices.map(d=>`<div class="list-item"><div class="link-row">${icon('device')}<div><strong>${esc(d.name)}</strong><p>${esc(d.os)} · ${esc(d.app)} · Último acceso: ${esc(d.last)}</p></div></div><div class="item-actions">${statusBadge(d)}${button('Revisar acceso',`device:${d.id}`,'quiet')}${next(d)}</div></div>`).join('')}${owner?'':'<p class="small subtle space-top">Solo la persona que administra la consulta puede revocar o autorizar equipos.</p>'}`;
  }
  return shell(head('Configuración','Preferencias y datos de tu consulta.')+`<div class="settings-layout"><nav class="settings-nav" aria-label="Secciones de configuración">${settingsItems.map(([name,id])=>`<a href="#/configuracion/${id}" ${section===id?'aria-current="page"':''}>${name}</a>`).join('')}</nav><div>${content}</div></div>`,'Configuración','configuracion');
}
function diagnostics(){return shell(head('Diagnóstico','Información de ejemplo para revisar la propuesta visual.')+notice('Esta pantalla no consulta el escritorio real.','Las versiones y el estado de servicios solo se verificarán en la aplicación Tauri.')+`<table class="data-table"><tbody><tr><td>Vista</td><td>Prototipo de diseño</td></tr><tr><td>Datos</td><td>Ficticios, en memoria</td></tr><tr><td>Zona horaria del ejemplo</td><td>América / Caracas</td></tr><tr><td>Conexiones a servicios</td><td>Ninguna</td></tr></tbody></table>`,'Ayuda');}

function auth(screen='acceso'){
  const setup=state.setup||{practice:'Consulta de Elena Arias',zone:'America/Caracas',location:'Centro Médico Los Olivos · Consultorio 204',service:'Consulta general',duration:'30',start:'09:00',end:'12:00',reviewed:false};
  if(screen==='preparacion'&&state.viewerRole==='assistant'&&setup.reviewed)return auth('preparacion/lista');
  let content='';
  if(screen==='preparacion'){
    content=`<h1>Prepara tu consulta</h1><p>Revisa los datos que necesitas para empezar. Podrás ajustarlos después en Configuración.</p><form id="setup-form"><fieldset class="form-section"><legend><span class="step-number">1</span> Consulta</legend><div class="fields"><label>Nombre de la consulta<input name="practice" value="${esc(setup.practice)}" required></label><label>Lugar de atención<input name="location" value="${esc(setup.location)}" required></label><label>Zona horaria<select name="zone"><option value="America/Caracas">Caracas · UTC −04:00</option></select></label></div></fieldset><fieldset class="form-section"><legend><span class="step-number">2</span> Primer servicio</legend><div class="two-fields"><label>Servicio<input name="service" value="${esc(setup.service)}" required></label><label>Duración de referencia · minutos<input type="number" name="duration" min="1" max="1440" value="${esc(setup.duration)}" required></label></div></fieldset><fieldset class="form-section"><legend><span class="step-number">3</span> Horario habitual</legend><p class="field-help">Lunes a viernes. Podrás añadir otros tramos y excepciones en Configuración.</p><div class="two-fields"><label>Desde<input name="start" type="time" value="${esc(setup.start)}" required></label><label>Hasta<input name="end" type="time" value="${esc(setup.end)}" required></label></div></fieldset><div id="form-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Guardar datos iniciales</button>${link('Volver','acceso/cuentas','button quiet')}</div></form>`;
    if(state.viewerRole==='assistant')content=`<h1>La consulta aún no está lista</h1><p>Elena Arias debe completar los datos iniciales. Tu acceso de asistente no permite cambiar esta configuración.</p><div class="access-context"><strong>Consulta de Elena Arias</strong><p>Cuenta autorizada · Asistente</p></div>${link('Volver a mis consultas','acceso/cuentas','button primary')}`;
  }else if(screen==='preparacion/lista'){
    content=setup.reviewed?`<h1>Tu consulta está lista</h1><p>Los datos iniciales se han guardado.</p><ul class="checklist setup-results"><li>${icon('check')}<div><strong>${esc(setup.practice)}</strong><p>${esc(setup.location)} · Hora de Caracas</p></div></li><li>${icon('check')}<div><strong>${esc(setup.service)}</strong><p>${esc(setup.duration)} minutos de referencia</p></div></li><li>${icon('check')}<div><strong>Lunes a viernes</strong><p>${esc(setup.start)} – ${esc(setup.end)}</p></div></li></ul>${link('Abrir Agenda','agenda','button primary')}${link('Revisar los datos','preparacion','button quiet')}`:`<h1>Falta preparar la consulta</h1><p>Completa y guarda los datos iniciales antes de abrirla por primera vez.</p>${link('Preparar consulta','preparacion','button primary')}`;
  }else{
    const states={
      acceso:['Entra a tu consulta','Gestiona tus citas y los datos de tus pacientes. El acceso continúa en tu navegador.','Continuar en el navegador','access-start'],
      'acceso/navegador':['Continúa en el navegador','Termina de iniciar sesión en la ventana del navegador y vuelve aquí. No compartas tu cuenta con otras personas.','Comprobar acceso','access-check'],
      'acceso/equipo':['Registra este equipo','Esta instalación todavía no tiene autorización para abrir la consulta. Necesitas conexión para registrarla.','Registrar este equipo','access-register'],
      'acceso/preparando':['Preparando tu consulta','Falta descargar la vista inicial de esta cuenta. Mantén la conexión para terminar la preparación.','Comprobar preparación','access-prepare'],
      'acceso/local':['Puedes abrir tu consulta','Este equipo conserva un acceso local válido y los datos ya disponibles. Puedes continuar con ese acceso.','Abrir Agenda','access-local'],
      'acceso/denegado':['No tienes acceso a esta consulta','El acceso de esta cuenta o equipo ya no está autorizado. Los datos guardados y el trabajo pendiente se conservan. Contacta con la persona que administra la consulta.','Volver a iniciar sesión','access-restart'],
      'acceso/conexion':['Necesitas conexión','Conecta este equipo para activar la consulta o descargar sus datos iniciales. Una instalación ya preparada puede conservar acceso local válido.','Volver a intentar','access-retry'],
      'acceso/servicio':['No pudimos comprobar el acceso','El servicio no está disponible en este momento. Tus datos guardados se conservan. Puedes volver a intentarlo.','Volver a intentar','access-retry'],
      'acceso/sesion-vencida':['Vuelve a iniciar sesión','La sesión del navegador ha vencido. Esto no borra los datos ni el trabajo guardado en el equipo.','Iniciar sesión de nuevo','access-start']
    };
    if(screen==='acceso/cuentas')content=`<h1>Abre tu consulta</h1><p>Consulta autorizada para esta cuenta.</p><div class="access-context"><strong>Consulta de Elena Arias</strong><p>${state.viewerRole==='assistant'?'Lucía Torres · Asistente':'Elena Arias · Dueña'}</p><span class="small subtle">Consulta independiente</span></div>${button('Continuar con esta consulta','access-account','primary full')}${link('Usar otra cuenta','acceso','button quiet full')}`;
    else{
      const [title,description,label,action]=states[screen]||states.acceso;
      content=`<h1>${title}</h1><p>${description}</p>${['acceso/equipo','acceso/preparando','acceso/local','acceso/denegado'].includes(screen)?'<div class="access-context"><strong>Consulta de Elena Arias</strong><p>Este equipo · Windows</p></div>':''}${screen==='acceso/preparando'?'<div class="access-progress" role="status">'+icon('clock')+' Preparación inicial pendiente</div>':''}${button(label,action,'primary full')}${screen==='acceso'?'':link('Volver al acceso','acceso','button quiet full')}${screen==='acceso/sesion-vencida'?link('Revisar acceso local','acceso/local','button quiet full'):''}`;
    }
  }
  return `<main id="main" class="auth-page"><section class="auth-card ${screen==='preparacion'?'setup-card':''}">${brand('acceso')}${content}</section></main>${prototypeBar()}`;
}

function hub(screen){
  if(screen==='cita'&&state.hubStatus==='Cancelada')screen='cancelada';
  const appointment=`<div class="hub-date"><div class="date-block"><strong>22</strong><small>OCTUBRE</small></div><div><strong>Jueves · 09:00 – 09:30</strong><p class="small muted">Hora de Caracas</p></div></div><div class="facts"><div class="fact"><span>Servicio</span><span>Consulta general</span></div><div class="fact"><span>Lugar</span><span>Centro Médico Los Olivos<br>Consultorio 204</span></div></div>`;
  let content='';
  if(screen==='cita')content=`<div class="practice-title">Dra. Elena Arias · Medicina general</div><h1>Tu próxima cita</h1><p>Hola, Ana María. Aquí puedes revisar los datos de tu cita.</p>${appointment}${badge(state.hubStatus)}<div class="hub-actions">${state.hubStatus==='Confirmada'?'':button(icon('check')+' Confirmar asistencia','hub-confirm','primary')}${link('Solicitar otra fecha','hub/cambiar','button')}${link('Cancelar mi cita','hub/cancelar','button quiet')}</div>`;
  if(screen==='confirmada')content=`<div class="hub-success">${icon('check')}</div><h1>Asistencia confirmada</h1><p>La consulta ha recibido tu confirmación. Te esperamos.</p>${appointment}<div class="hub-actions">${link('Volver a mi cita','hub/cita','button')}</div>`;
  if(screen==='cancelar')content=`<h1>Cancelar tu cita</h1><p>Revisa los datos antes de continuar.</p>${appointment}<p>Al cancelar, este horario podrá ofrecerse a otra persona. La cita quedará cancelada cuando recibas la confirmación.</p><div class="hub-actions">${button('Confirmar cancelación','hub-cancel','primary')}${link('Conservar mi cita','hub/cita','button')}</div>`;
  if(screen==='cancelada')content=`<div class="hub-success">${icon('check')}</div><h1>Tu cita se ha cancelado</h1><p>La consulta ha recibido la cancelación. Ya no necesitas asistir el 22 de octubre a las 09:00.</p><div class="section-line"><p>Para registrar otra cita, contacta con la consulta.</p></div>`;
  if(screen==='cambiar')content=`<h1>¿Necesitas otra fecha?</h1><p>Envía tu solicitud. La consulta revisará el cambio contigo.</p>${appointment}<form id="hub-request-form"><label>¿Hay algo que debamos saber? <span class="small subtle">Opcional</span><textarea name="reason" placeholder="Por ejemplo, prefiero otro día por la mañana."></textarea></label><p class="field-help">Incluye solo información sobre el horario, sin datos médicos.</p><div class="hub-actions"><button type="submit" class="primary">Enviar solicitud</button>${link('Volver a mi cita','hub/cita','button quiet')}</div></form>`;
  if(screen==='solicitud')content=`<div class="hub-success">${icon('check')}</div><h1>Solicitud recibida</h1><p>La consulta revisará tu solicitud y se pondrá en contacto contigo.</p><div class="section-line"><strong>Tu cita conserva su horario</strong><p style="margin-top:8px">22 de octubre · 09:00 – 09:30. Pedir un cambio no reserva otra hora.</p></div><div class="hub-actions">${link('Ver mi cita','hub/cita','button')}</div>`;
  if(screen==='enlace-vencido')content=`<div class="hub-success" style="background:var(--panel);color:var(--muted)">${icon('link')}</div><h1>Este enlace ya no está disponible</h1><p>Puede haber vencido o la consulta lo ha desactivado.</p><div class="section-line"><p>Pide a la consulta un nuevo enlace para revisar tu cita.</p></div>`;
  if(screen==='no-disponible')content=`<div class="hub-success" style="background:var(--panel);color:var(--muted)">${icon('clock')}</div><h1>No pudimos comprobar el resultado</h1><p>No sabemos todavía si la acción se completó. Conservaremos la solicitud para revisar su resultado.</p><div class="hub-actions">${button('Volver a comprobar','hub-retry','primary')}${link('Volver a mi cita','hub/cita','button quiet')}</div>`;
  if(screen==='servicio')content=`<div class="hub-success" style="background:var(--panel);color:var(--muted)">${icon('info')}</div><h1>No pudimos mostrar tu cita</h1><p>El servicio no está disponible en este momento. Tus datos no se han perdido; inténtalo de nuevo en unos minutos.</p><div class="hub-actions">${button('Volver a intentar','hub-service-retry','primary')}</div>`;
  if(screen==='cita-modificada')content=`<div class="hub-success" style="background:var(--panel);color:var(--muted)">${icon('calendar')}</div><h1>Tu cita ha cambiado</h1><p>La consulta movió tu cita. Este es el horario que te corresponde ahora.</p><div class="hub-date"><div class="date-block"><strong>23</strong><small>OCTUBRE</small></div><div><strong>Viernes · 09:30 – 10:00</strong><p class="small muted">Hora de Caracas</p></div></div><div class="facts"><div class="fact"><span>Servicio</span><span>Consulta general</span></div><div class="fact"><span>Lugar</span><span>Centro Médico Los Olivos<br>Consultorio 204</span></div></div><div class="section-line"><p class="small muted">Horario anterior: jueves 22 de octubre, 09:00 – 09:30.</p></div><div class="hub-actions">${link('Volver a mi cita','hub/cita','button')}</div>`;
  return `<main id="main" class="hub-page"><div class="hub-brand">${brand('hub/cita')}</div><section class="hub-card">${content}</section><p class="hub-foot">Solo la información administrativa de tu cita.<br>Prototipo: las acciones y respuestas son simuladas.</p></main>${prototypeBar()}`;
}
function catalog(){return `<main id="main" class="catalog">${brand()}<h1>Pantallas de Nightingale</h1><p>Un recorrido por la propuesta de escritorio y el acceso del paciente. Todas las pantallas comparten el mismo sistema visual y datos de ejemplo.</p><div class="head-actions space-bottom">${link('Explorar Agenda '+icon('arrow'),'agenda','button primary')}${link('Abrir vista del paciente','hub/cita','button')}</div><div class="catalog-grid">${screens.map(([group,items])=>`<section class="catalog-group"><h2>${group}</h2>${items.map(([path,title])=>link(esc(title)+icon('right'),path)).join('')}</section>`).join('')}</div></main>${prototypeBar()}`;}

let datePicker=null,dateInput=null,dateDraft='',dateMonth='';
function enhanceDates(){
  if(!('showPopover' in HTMLElement.prototype))return;
  document.querySelectorAll('input[type="date"]').forEach((input,index)=>{
    if(input.closest('.date-field'))return;
    if(!input.id)input.id=`date-input-${index}`;
    const labelElement=input.closest('label');
    const label=labelElement?.textContent.trim()||input.getAttribute('aria-label')||'Fecha';
    const field=document.createElement('span');field.className='date-field';
    input.before(field);field.append(input);
    if(labelElement){
      const group=document.createElement('div');group.className='date-label';
      labelElement.before(group);labelElement.htmlFor=input.id;group.append(labelElement,field);
    }
    const trigger=document.createElement('button');trigger.type='button';trigger.className='date-trigger';
    trigger.innerHTML=icon('calendar');trigger.setAttribute('aria-label',`Elegir fecha: ${label}`);trigger.setAttribute('aria-haspopup','dialog');trigger.setAttribute('aria-expanded','false');
    trigger.title='Abrir calendario';
    trigger.disabled=input.disabled||input.readOnly||!!input.closest('fieldset:disabled');
    trigger.addEventListener('click',()=>openDatePicker(input,trigger));field.append(trigger);
  });
}
function closeDatePicker(restore=true){
  if(!datePicker)return;
  const trigger=dateInput?.closest('.date-field')?.querySelector('button');
  trigger?.setAttribute('aria-expanded','false');
  datePicker.remove();datePicker=null;
  if(restore)trigger?.focus({preventScroll:true});
}
function openDatePicker(input,trigger){
  closeDatePicker(false);dateInput=input;dateDraft=input.value||TODAY;dateMonth=dateDraft.slice(0,7);
  datePicker=document.createElement('div');datePicker.className='date-picker';datePicker.id='date-picker';
  datePicker.setAttribute('popover','auto');datePicker.setAttribute('role','dialog');datePicker.setAttribute('aria-label','Elegir fecha');
  // Popovers inside a modal must belong to that modal's interactive subtree.
  // https://developer.mozilla.org/en-US/docs/Web/API/Popover_API/Using
  (input.closest('dialog')||document.body).append(datePicker);
  trigger.setAttribute('aria-controls','date-picker');trigger.setAttribute('aria-expanded','true');
  datePicker.addEventListener('toggle',event=>{if(event.newState==='closed'&&datePicker===event.target)closeDatePicker();});
  datePicker.addEventListener('keydown',event=>{
    const day=event.target.closest('[data-date]');
    if(!day)return;
    const dayDate=day.dataset.date;
    const offset={ArrowLeft:-1,ArrowRight:1,ArrowUp:-7,ArrowDown:7,Home:-((new Date(dayDate+'T12:00:00Z').getUTCDay()+6)%7),End:6-((new Date(dayDate+'T12:00:00Z').getUTCDay()+6)%7)}[event.key];
    if(offset!==undefined){event.preventDefault();const next=shiftedDate(dayDate,offset);dateMonth=next.slice(0,7);paintDatePicker(next);}
    else if(['PageUp','PageDown'].includes(event.key)){event.preventDefault();moveDateMonth(event.key==='PageUp'?-1:1);}
  });
  paintDatePicker(dateDraft);datePicker.showPopover();positionDatePicker();
  (datePicker.querySelector('[data-date][tabindex="0"]')||datePicker.querySelector('select')).focus();
}
function positionDatePicker(){
  if(!datePicker)return;
  const rect=dateInput.getBoundingClientRect(),height=datePicker.offsetHeight;
  datePicker.style.left=Math.max(12,Math.min(rect.left,document.documentElement.clientWidth-datePicker.offsetWidth-12))+'px';
  datePicker.style.top=Math.max(12,Math.min(rect.bottom+6+height<=innerHeight-12?rect.bottom+6:rect.top-height-6,innerHeight-height-12))+'px';
}
function moveDateMonth(offset){
  const date=new Date(dateMonth+'-01T12:00:00Z');date.setUTCMonth(date.getUTCMonth()+offset);dateMonth=date.toISOString().slice(0,7);paintDatePicker(dateMonth+'-01');
}
function paintDatePicker(focusDate=''){
  const [year,month]=dateMonth.split('-').map(Number),first=new Date(dateMonth+'-01T12:00:00Z');
  const leading=(first.getUTCDay()+6)%7,count=new Date(Date.UTC(year,month,0)).getUTCDate();
  const available=value=>(!dateInput.min||value>=dateInput.min)&&(!dateInput.max||value<=dateInput.max);
  const days=Array.from({length:Math.ceil((leading+count)/7)*7},(_,index)=>{
    const number=index-leading+1;if(number<1||number>count)return'<td></td>';
    const value=`${dateMonth}-${String(number).padStart(2,'0')}`;
    return `<td role="gridcell" aria-selected="${value===dateDraft}"><button type="button" data-date="${value}" aria-label="${esc(fullDate(value))}" ${value===TODAY?'aria-current="date"':''} tabindex="${value===(focusDate||dateDraft)?'0':'-1'}" ${available(value)?'':'disabled'}>${number}</button></td>`;
  });
  const rows=Array.from({length:days.length/7},(_,i)=>`<tr>${days.slice(i*7,i*7+7).join('')}</tr>`).join('');
  datePicker.innerHTML=`<div class="date-summary" aria-live="polite"><small>Fecha seleccionada</small><strong>${esc(titleCase(fullDate(dateDraft)))}, ${dateDraft.slice(0,4)}</strong></div><div class="date-picker-nav"><select aria-label="Mes">${Array.from({length:12},(_,i)=>`<option value="${i+1}" ${i+1===month?'selected':''}>${titleCase(new Intl.DateTimeFormat('es',{month:'long',timeZone:'UTC'}).format(new Date(Date.UTC(year,i,1))))}</option>`).join('')}</select><select aria-label="Año">${Array.from({length:Math.max(year+1,Number(TODAY.slice(0,4))+20)-Math.min(1900,year-1)+1},(_,i)=>{const y=Math.min(1900,year-1)+i;return `<option ${y===year?'selected':''}>${y}</option>`;}).join('')}</select><button type="button" class="quiet" data-month="-1" aria-label="Mes anterior">${icon('left')}</button><button type="button" class="quiet" data-month="1" aria-label="Mes siguiente">${icon('right')}</button></div><table role="grid" aria-label="${titleCase(new Intl.DateTimeFormat('es',{month:'long',year:'numeric',timeZone:'UTC'}).format(first))}"><thead><tr>${[1,2,3,4,5,6,0].map(d=>`<th scope="col" abbr="${dayNames[d]}">${dayNames[d].slice(0,2)}</th>`).join('')}</tr></thead><tbody>${rows}</tbody></table><p class="date-picker-help">Flechas para moverte · Esc para cerrar</p><div class="date-picker-footer"><button type="button" class="quiet" data-calendar="today" ${available(TODAY)?'':'disabled'}>Hoy</button><button type="button" class="quiet" data-calendar="cancel">Cancelar</button><button type="button" class="primary" data-calendar="apply" ${available(dateDraft)?'':'disabled'}>Usar fecha ${icon('check')}</button></div>`;
  datePicker.querySelectorAll('[data-date]').forEach(day=>day.addEventListener('click',()=>{dateDraft=day.dataset.date;paintDatePicker(dateDraft);}));
  datePicker.querySelectorAll('[data-month]').forEach(button=>button.addEventListener('click',()=>moveDateMonth(Number(button.dataset.month))));
  datePicker.querySelector('[aria-label="Mes"]').addEventListener('change',event=>{dateMonth=`${year}-${String(event.target.value).padStart(2,'0')}`;paintDatePicker(dateMonth+'-01');});
  datePicker.querySelector('[aria-label="Año"]').addEventListener('change',event=>{dateMonth=`${event.target.value}-${String(month).padStart(2,'0')}`;paintDatePicker(dateMonth+'-01');});
  datePicker.querySelector('[data-calendar="today"]').addEventListener('click',()=>{dateDraft=TODAY;dateMonth=TODAY.slice(0,7);paintDatePicker(dateDraft);});
  datePicker.querySelector('[data-calendar="cancel"]').addEventListener('click',()=>closeDatePicker());
  datePicker.querySelector('[data-calendar="apply"]').addEventListener('click',()=>{
    const input=dateInput,id=input.id,value=dateDraft;closeDatePicker(false);input.value=value;
    input.dispatchEvent(new Event('input',{bubbles:true}));input.dispatchEvent(new Event('change',{bubbles:true}));
    const replacement=document.getElementById(id);replacement?.closest('.date-field')?.querySelector('button').focus({preventScroll:true});
  });
  if(!datePicker.querySelector('[data-date][tabindex="0"]:not(:disabled)'))datePicker.querySelector('[data-date]:not(:disabled)')?.setAttribute('tabindex','0');
  if(datePicker.matches(':popover-open')){positionDatePicker();datePicker.querySelector('[data-date][tabindex="0"]:not(:disabled)')?.focus();}
}
window.addEventListener('resize',()=>closeDatePicker());
document.addEventListener('scroll',event=>{if(datePicker&&!datePicker.contains(event.target))positionDatePicker();},true);

function render(){
  closeDatePicker(false);
  const previousRoute=route;
  const focused=document.activeElement?.closest('select')||document.activeElement;
  const focusId=focused?.id;
  const focusAction=focused?.getAttribute('data-action');
  route=(location.hash.replace(/^#\/?/,'')||'agenda').split('?')[0];
  if(['agenda','agenda/semana','citas'].includes(route))state.agendaView=route;
  const detailId=/^cita\/(?!nueva$)[^/]+$/.test(route)?route.split('/')[1]:null;
  const appointment=detailId?state.appointments.find(a=>a.id===detailId):null;
  const surfaceRoute=appointment?state.detailOrigin||state.agendaView:route;
  const appointmentDialog=document.getElementById('appointment-dialog');
  if(!appointment&&appointmentDialog.open)appointmentDialog.close();
  let html='';
  if(surfaceRoute==='pantallas')html=catalog();
  else if(surfaceRoute==='acceso'||surfaceRoute.startsWith('acceso/')||surfaceRoute==='preparacion'||surfaceRoute.startsWith('preparacion/'))html=auth(surfaceRoute);
  else if(surfaceRoute.startsWith('hub/'))html=hub(surfaceRoute.split('/')[1]);
  else if(surfaceRoute==='agenda/semana')html=week();
  else if(surfaceRoute==='citas')html=appointmentsList();
  else if(route==='cita/nueva')html=appointmentEditor();
  else if(/^cita\/[^/]+\/reprogramar$/.test(route))html=appointmentEditor(route.split('/')[1]);
  else if(detailId&&!appointment)html=shell(head('Cita no disponible')+empty('No encontramos esta cita','Vuelve a la agenda para elegir una cita disponible.',link('Volver a Agenda',state.agendaView,'button')),'Agenda');
  else if(route==='pacientes/nuevo')html=patientForm();
  else if(surfaceRoute==='pacientes')html=patients();
  else if(surfaceRoute.startsWith('paciente/'))html=patientRecord(surfaceRoute.split('/')[1]);
  else if(surfaceRoute==='solicitudes')html=requests();
  else if(surfaceRoute.startsWith('solicitud/'))html=requestDetail(surfaceRoute.split('/')[1]);
  else if(route==='lista-espera')html=waitlist();
  else if(route==='comunicaciones')html=communications();
  else if(route==='indicadores')html=metrics();
  else if(route==='actividad')html=activity();
  else if(route.startsWith('conflicto/'))html=conflict();
  else if(route.startsWith('configuracion/'))html=settings(route.split('/')[1]);
  else if(route==='diagnostico')html=diagnostics();
  else html=agenda();
  document.getElementById('app').innerHTML=html;
  enhanceDates();
  const title=appointment?'Cita de '+getPatient(appointment.patient).name:document.querySelector('h1')?.textContent||'Agenda';
  document.title=`Nightingale · ${title}`;
  renderedHash=location.hash||href(route);
  if(appointment){
    appointmentDialog.innerHTML=appointmentDetail(appointment);
    // Native modal handling keeps the background inert, including under confirmation dialogs.
    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/showModal
    if(!appointmentDialog.open)appointmentDialog.showModal();
    const control=focusAction?Array.from(appointmentDialog.querySelectorAll('[data-action]')).find(el=>el.dataset.action===focusAction):null;
    (control||document.getElementById('appointment-title')).focus({preventScroll:true});
    return;
  }
  if(previousRoute&&previousRoute!==route){
    const returnLink=route===state.detailOrigin&&/^cita\/[^/]+$/.test(previousRoute)?Array.from(document.querySelectorAll('.data-table a,.week-event,a[href^="#/cita/"]')).find(a=>a.getAttribute('href')===href(previousRoute)):null;
    const target=returnLink||document.querySelector('.detail-panel h2')||document.querySelector('main h1,main h2');
    if(target){if(!returnLink)target.setAttribute('tabindex','-1');target.focus({preventScroll:true});}
  }else{
    const target=focusId?document.getElementById(focusId):focusAction?Array.from(document.querySelectorAll('[data-action]')).find(el=>el.dataset.action===focusAction):null;
    target?.focus({preventScroll:true});
  }
}
function toggleNavigation(open){
  const sidebar=document.getElementById('sidebar');
  sidebar.classList.toggle('is-open',open);
  const trigger=document.querySelector('[data-action="menu"]');
  trigger.setAttribute('aria-expanded',String(open));
  // Keep the drawer's background outside keyboard navigation while it is open.
  // https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/inert
  document.querySelector('.workspace').inert=open;
  document.querySelector('.prototype-bar').inert=open;
  if(open){sidebar.setAttribute('role','dialog');sidebar.setAttribute('aria-modal','true');sidebar.querySelector('[data-action="close-menu"]').focus();}
  else{sidebar.removeAttribute('role');sidebar.removeAttribute('aria-modal');trigger.focus({preventScroll:true});}
}
function go(path){location.hash=href(path);}
function toast(text){const el=document.getElementById('toast');el.textContent=text;el.classList.add('visible');clearTimeout(toastTimer);toastTimer=setTimeout(()=>el.classList.remove('visible'),4500);}
function dialog(title,body,actions=''){const booking=document.getElementById('appointment-form');if(booking&&canEditAppointments())activeDraft=formValues(booking);const el=document.getElementById('dialog');el.innerHTML=`<h2 id="dialog-title">${title}</h2>${body}<div class="dialog-actions">${button('Cerrar','close-dialog')}${actions}</div>`;if(!el.open)el.showModal();enhanceDates();}
function closeDialog(){closeDatePicker(false);document.getElementById('dialog').close();pendingNavigation=null;}
function errorMessage(text){const el=document.getElementById('form-error');el.textContent=text;el.setAttribute('tabindex','-1');el.focus();el.scrollIntoView({block:'nearest'});}
function dialogError(text){const el=document.getElementById('dialog-error');if(!el)return errorMessage(text);el.textContent=text;el.setAttribute('tabindex','-1');el.focus();}
function formValues(form){return Object.fromEntries(new FormData(form));}
function editPatient(id){const p=getPatient(id);dialog('Editar información',`<form id="edit-patient-form" data-id="${id}">${patientIdentityFields(p)}${patientContactFields(p)}<div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Guardar cambios</button></div></form>`);}
function searchDialog(){dialog('Buscar en la consulta',`<input id="global-search" type="search" aria-label="Paciente o cita" placeholder="Nombre, hora, servicio o fecha…" value="${esc(state.lastSearch||'')}" autofocus><div class="search-results" id="global-results">${searchResults(state.lastSearch||'')}</div>`);}
function searchResults(query){
  const q=query.toLocaleLowerCase('es').trim();
  if(!q)return '<p class="small muted">Busca por nombre de paciente o por hora, servicio o fecha de una cita.</p>';
  const patients=state.patients.filter(p=>p.name.toLocaleLowerCase('es').includes(q)||(p.phoneNumber||'').includes(q));
  const appts=state.appointments.filter(a=>{const p=getPatient(a.patient);return a.start.includes(q)||a.end.includes(q)||a.service.toLocaleLowerCase('es').includes(q)||a.date.includes(q)||p.name.toLocaleLowerCase('es').includes(q);}).slice(0,8);
  const group=(title,rows)=>rows?`<p class="small nav-label" style="margin-top:12px">${title}</p>${rows}`:'';
  const rows=group('Pacientes',patients.map(p=>`<a href="#/paciente/${p.id}" data-close="true"><span class="link-row">${avatar(p.name,p.id)}${esc(p.name)}</span><span class="small subtle">Paciente · ${esc(phoneText(p)||'No registrado')}</span></a>`).join(''))+group('Citas',appts.map(a=>{const p=getPatient(a.patient);return `<a href="#/cita/${a.id}" data-close="true"><span class="link-row">${esc(p.name)}</span><span class="small subtle">${fmt(a.date)}, ${a.start} – ${a.end} · ${esc(a.service)}</span></a>`;}).join(''));
  return rows||'<p class="small muted">No hay coincidencias en los datos locales del ejemplo.</p>';
}

document.addEventListener('click',event=>{
  const el=event.target.closest('[data-action]');
  if(event.target.closest('[data-close]'))closeDialog();
  if(!el)return;
  const [action,id]=el.dataset.action.split(':');
  if(action==='close-dialog')closeDialog();
  else if(action==='close-appointment')go(state.detailOrigin||state.agendaView);
  else if(action==='palette')dialog('Colores propuestos',`<p>Propuesta para revisar. El color distingue estados y ayuda a reconocer personas; el texto conserva su significado.</p><div class="palette-list"><div><span class="palette-swatch scheduled"></span><div><strong>Azul · Programada</strong><p>Identifica las citas aún sin confirmar. También conserva su uso en acciones, selección y foco.</p></div></div><div><span class="palette-swatch success"></span><div><strong>Verde · Confirmada</strong><p>Asistencia confirmada o comunicación entregada.</p></div></div><div><span class="palette-swatch attention"></span><div><strong>Ámbar · Requiere atención</strong><p>Ausencia, ficha por completar o solicitud por revisar.</p></div></div><div><span class="palette-swatch cancelled"></span><div><strong>Rosa · Cancelada</strong><p>Distingue una cancelación sin tratarla como una alerta urgente.</p></div></div><div><span class="palette-swatch neutral"></span><div><strong>Neutro · Atendida y datos habituales</strong><p>Conserva legibles la historia y los datos que no requieren una decisión.</p></div></div></div><section class="palette-identity"><strong>Iniciales de personas</strong><p>Fondos suaves con letras oscuras. Cada persona conserva su color; estos tonos no indican el estado de sus citas.</p><div class="palette-avatars">${state.patients.slice(0,6).map(p=>avatar(p.name,p.id)).join('')}</div></section><p class="small subtle">Los nuevos tonos son una propuesta. La paleta aprobada no cambia hasta tu revisión.</p>`);
  else if(action==='access-start'||action==='access-restart')go(state.scenario==='offline'?'acceso/conexion':state.scenario==='unavailable'?'acceso/servicio':'acceso/navegador');
  else if(action==='access-check')go(state.scenario==='offline'?'acceso/conexion':state.scenario==='unavailable'?'acceso/servicio':'acceso/cuentas');
  else if(action==='access-account')go(state.scenario==='offline'?'acceso/conexion':'acceso/equipo');
  else if(action==='access-register')go(state.scenario==='offline'?'acceso/conexion':state.scenario==='unavailable'?'acceso/servicio':'acceso/preparando');
  else if(action==='access-prepare')go(state.scenario==='offline'?'acceso/conexion':state.scenario==='unavailable'?'acceso/servicio':state.setup?.reviewed?'preparacion/lista':'preparacion');
  else if(action==='access-local')go('agenda');
  else if(action==='access-retry'){if(['offline','unavailable'].includes(state.scenario))toast(state.scenario==='offline'?'Este equipo sigue sin conexión en el ejemplo.':'El servicio sigue sin estar disponible en el ejemplo.');else go('acceso/navegador');}
  else if(action==='menu')toggleNavigation(true);
  else if(action==='close-menu')toggleNavigation(false);
  else if(action==='today'){state.date=TODAY;render();}
  else if(action==='day'){state.date=id;go('agenda');}
  else if(action==='agenda-retry'){state.agendaMode='ready';render();}
  else if(action==='more-appointments'){state.listLimit+=8;render();}
  else if(action==='date'){const date=new Date(state.date+'T12:00:00Z');date.setUTCDate(date.getUTCDate()+Number(id));state.date=date.toISOString().slice(0,10);render();}
  else if(action==='clear-filter'){state.filter='';state.professionalFilter='';state.serviceFilter='';state.appointmentQuery='';state.includeCancelled=false;state.listLimit=8;render();}
  else if(action==='clear-search'){state.query='';state.filter='';state.professionalFilter='';render();}
  else if(action==='search')searchDialog();
  else if(action==='help')dialog('Ayuda de Nightingale',`<p class="small muted">Guía breve por tarea; los enlaces abren la pantalla correspondiente del ejemplo.</p><div class="section-line"><h3>Cambiar o cancelar una cita</h3><p class="small">Abre la cita desde la Agenda y usa «Reprogramar» o «Más acciones». El paciente conserva sus datos.</p>${link('Ir a la Agenda','agenda','button quiet')}</div><div class="section-line"><h3>Registrar un paciente</h3><p class="small">Desde Pacientes, «Registrar paciente»; el formulario distingue adulto, menor y borrador.</p>${link('Ir a Pacientes','pacientes','button quiet')}</div><div class="section-line"><h3>Revisar solicitudes del paciente</h3><p class="small">En Solicitudes, elige un nuevo horario desde la solicitud; se resuelve al guardar, sin duplicar cambios.</p>${link('Ir a Solicitudes','solicitudes','button quiet')}</div><div class="section-line"><h3>Asignar un horario desde la lista de espera</h3><p class="small">Ofrece un intervalo compatible y asígnalo; la confirmación la da el personal.</p>${link('Ir a Lista de espera','lista-espera','button quiet')}</div>`,link('Ver todas las pantallas','pantallas','button')+link('Diagnóstico','diagnostico','button'));
  else if(action==='filters')dialog('Filtrar la agenda',`<p>Los filtros no cambian la disponibilidad.</p><div class="fields"><label>Servicio<select id="dialog-service"><option value="">Todos los servicios</option>${options(offeredNames(),state.serviceFilter)}</select></label><label class="check-line"><input id="dialog-cancelled" type="checkbox" ${state.includeCancelled?'checked':''}>Incluir citas canceladas</label></div>`,button('Aplicar filtros','apply-filter','primary'));
  else if(action==='apply-filter'){state.serviceFilter=document.getElementById('dialog-service').value;state.includeCancelled=document.getElementById('dialog-cancelled').checked;state.listLimit=8;closeDialog();render();}
  else if(action==='appointment-menu'){const a=getAppointment(id);dialog(getPatient(a.patient).name,`<p>${fmt(a.date)} · ${a.start} – ${a.end}<br>${esc(a.service)}</p><div class="detail-actions">${link('Ver cita',`cita/${id}`,'button')}${actionAllowed(a,'reschedule')?link('Reprogramar',`cita/${id}/reprogramar`,'button'):''}</div>`,actionAllowed(a,'cancel')?button('Cancelar cita',`cancel:${id}`):'');}
  else if(action==='cancel'){const a=getAppointment(id);if(!actionAllowed(a,'cancel')){toast('Esta cita no se puede cancelar.');return;}if(document.getElementById('dialog').open)closeDialog();dialog('¿Cancelar esta cita?',`<p><strong>${esc(getPatient(a.patient).name)}</strong><br>${fmt(a.date)} · ${a.start} – ${a.end}<br>${esc(a.service)}</p><p>Se conservará la historia. El horario podrá ofrecerse a otra persona cuando se acepte la cancelación.</p>`,button('Confirmar cancelación',`do-cancel:${id}`,'primary'));}
  else if(action==='do-cancel')changeAppointment(id,'cancel');
  else if(action==='absence'){const a=getAppointment(id);if(!actionAllowed(a,'absence')){toast('Todavía no se puede registrar ausencia.');return;}dialog('¿El paciente no asistió?',`<p><strong>${esc(getPatient(a.patient).name)}</strong><br>${fmt(a.date)} · ${a.start} – ${a.end}</p><p>Confirma que el paciente no asistió. Esta acción termina la cita y conserva su historia.</p>`,button('Confirmar ausencia',`do-absence:${id}`,'primary'));}
  else if(action==='do-absence')changeAppointment(id,'absence');
  else if(action==='confirm'||action==='complete')changeAppointment(id,action);
  else if(action==='edit-patient')editPatient(id);
  else if(action==='save-patient-draft'){const form=document.getElementById('patient-form');form.dataset.saveAs='draft';form.requestSubmit();}
  else if(action==='add-responsible')addResponsible(id);
  else if(action==='edit-responsible'){const [,cid,pid]=el.dataset.action.split(':');editResponsible(cid,pid);}
  else if(action==='deactivate-responsible'){const [,cid]=el.dataset.action.split(':');const c=responsibleContacts.find(c=>c.id===cid);const conflicts=responsibleConflicts(c);if(conflicts.length)dialog('No se puede inactivar este vínculo',`<p>${esc(c.name)} es el único responsable vigente con teléfono de ${conflicts.map(p=>esc(p.name)).join(' y ')}. Inactivarlo dejaría una ficha completa sin sus requisitos.</p><p>Añade otro responsable vigente o completa el teléfono del paciente antes. Los datos no se han cambiado.</p>`);else dialog('¿Inactivar el vínculo con '+esc(c.name)+'?','<p>Dejará de aparecer como responsable vigente. Los datos se conservan y esto no cancela citas.</p>',button('Inactivar vínculo',`do-deactivate-responsible:${cid}`,'primary'));}
  else if(action==='do-deactivate-responsible'){const [,cid]=el.dataset.action.split(':');const c=responsibleContacts.find(c=>c.id===cid);c.active=false;c.until=TODAY;closeDialog();render();toast('Vínculo inactivado en el ejemplo.');}
  else if(action==='reactivate-responsible'){const [,cid]=el.dataset.action.split(':');const c=responsibleContacts.find(c=>c.id===cid);c.active=true;delete c.until;render();toast('Vínculo reactivado en el ejemplo.');}
  else if(action==='toggle-patient'){const p=getPatient(id);if(p.active===false){p.active=true;render();toast('Ficha reactivada en el ejemplo.');}else dialog('¿Inactivar la ficha de '+esc(p.name)+'?','<p>Inactivar no cancela sus citas: los encuentros programados siguen registrados y se conservan sus datos.</p><p>La ficha dejará de aparecer al crear citas nuevas hasta que se reactive.</p>',button('Inactivar ficha',`do-toggle-patient:${id}`,'primary'));}
  else if(action==='do-toggle-patient'){const p=getPatient(id);p.active=false;closeDialog();render();toast('Ficha inactivada en el ejemplo. Las citas no se cancelan.');}
  else if(action==='more-patient-history'){state.patientHistory=(state.patientHistory||5)+5;render();}
  else if(action==='inline-patient')registerInlinePatient();
  else if(action==='inline-responsible')dialog('Registrar responsable',`<form id="inline-responsible-form" data-patient="${document.getElementById('appointment-form').elements.patient.value}">${responsibleFields()}<div id="inline-responsible-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Registrar y elegir</button></div></form>`);
  else if(action==='choose-slot'){const form=document.getElementById('appointment-form');form.elements.start.value=el.dataset.start;form.elements.end.value=el.dataset.end;form.dataset.dirty='true';activeDraft=formValues(form);render();}
  else if(action==='add-waitlist')dialog('Añadir a la lista de espera',`<form id="waitlist-form"><div class="fields"><label>Paciente<select name="patient">${state.patients.filter(p=>p.active!==false).map(p=>`<option value="${p.id}">${esc(p.name)}</option>`).join('')}</select></label><label>Servicio<select name="service">${options(offeredNames())}</select></label><label>Preferencia de horario<input name="preference" placeholder="Por ejemplo, por las mañanas"></label></div><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Añadir paciente</button></div></form>`);
  else if(action==='edit-waitlist'){
    const w=state.waitlist.find(w=>w.id===id);
    if(!w)return;
    const p=getPatient(w.patient),services=offeredNames();
    if(!services.includes(w.service))services.push(w.service);
    dialog('Editar entrada de '+esc(p.name),`<form id="waitlist-form" data-waitlist="${w.id}"><div class="fields"><p class="muted small">${esc(p.name)} · En espera desde ${fmt(w.since)}</p><label>Servicio<select name="service">${options(services,w.service)}</select></label><label>Preferencia de horario<input name="preference" value="${esc(w.preference)}"></label></div><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Guardar cambios</button></div></form>`);
  }
  else if(action==='offer-candidates'){
    const w=state.waitlist.find(w=>w.id===id);
    if(!w||w.assigned)return;
    const p=getPatient(w.patient);
    const slots=[];
    for(let i=0;i<5&&slots.length<2;i++){const date=shiftedDate(TODAY,i);const slot=availableSlot(date,'elena',referenceDuration(w.service));if(slot)slots.push({date,slot});}
    dialog('Horarios compatibles para '+esc(p.name),`<p>${esc(w.service)} · ${esc(w.preference)}</p><p class="small muted">Solo aparecen intervalos compatibles con el horario y la ocupación conocida.</p>${slots.length?slots.map(({date,slot})=>`<div class="list-item"><div><strong>${titleCase(fullDate(date))}</strong><p>${slot.start} – ${slot.end}</p></div>${button('Asignar este horario',`assign-slot:${w.id}`,'button',`data-date="${date}" data-start="${slot.start}" data-end="${slot.end}"`)}</div>`).join(''):'<p class="muted small">No hay intervalos compatibles en los próximos días del ejemplo.</p>'}<p class="small subtle">La asignación la confirma el personal; la API volvería a comprobar disponibilidad antes de registrar la cita.</p>`);
  }
  else if(action==='assign-slot'){
    const w=state.waitlist.find(w=>w.id===id);
    if(!w||w.assigned)return;
    const {date,start,end}=el.dataset;
    const p=getPatient(w.patient);
    dialog('¿Asignar este horario a '+esc(p.name)+'?',`<p>${esc(w.service)} · ${fmt(date)} · ${start} – ${end}</p><p>Crea la cita en el ejemplo tras una aceptación ilustrada del personal. La API volvería a comprobar disponibilidad antes de confirmar.</p>`,button('Asignar y crear cita',`do-assign-slot:${w.id}`,'primary',`data-date="${date}" data-start="${start}" data-end="${end}"`));
  }
  else if(action==='do-assign-slot'){
    const w=state.waitlist.find(w=>w.id===id);
    if(!w)return;
    const {date,start,end}=el.dataset;
    const occupied=occupiedIntervals(date,'elena').some(a=>a.start<end&&a.end>start);
    if(occupied){closeDialog();render();toast('Ese horario ya no está disponible; la entrada se conserva en la lista de espera.');return;}
    const p=getPatient(w.patient);
    const saved={id:'a'+(state.appointments.length+1),patient:p.id,professionalId:'elena',service:w.service,date,start,end,responsibleContactId:'',status:'Programada',location:accountLocation(),professionalSnapshot:professionalName('elena'),history:[],localActivity:[],accepted:null,created:true};
    state.appointments.push(saved);
    recordAppointmentChange(saved,'Cita asignada desde la lista de espera');
    w.assigned=true;
    closeDialog();render();toast('Cita creada en el ejemplo para '+esc(p.name)+'.');
  }
  else if(action==='message'){
    const m=state.messages.find(m=>m.id===id);
    if(!m)return;
    const p=getPatient(m.patient),a=getAppointment(m.appointmentId);
    const review=(m.status==='Revisar teléfono'||m.status==='Resultado desconocido')?button('Revisar contacto',`review-contact:${m.id}`,'primary'):'';
    dialog('Detalle de comunicación',`<p><strong>${esc(p.name)}</strong><br>${esc(m.type)} · ${esc(m.channel)}</p><p class="muted">${a?`Cita: ${fmt(a.date)}, ${a.start} – ${a.end} · ${esc(a.service)}`:'Sin cita relacionada'}</p><p class="muted">Envío: ${fmt(m.date)} · ${m.time}</p><p>Estado: ${badge(m.status)}</p><p>${esc(m.text)}</p><p class="small subtle">Las comunicaciones no cambian por sí solas el estado de la cita.</p>`,review+link('Ver cita',`cita/${a.id}`,'button')+link('Ver paciente',`paciente/${p.id}`,'button'));
  }
  else if(action==='review-contact'){
    const m=state.messages.find(m=>m.id===id);
    if(!m)return;
    const p=getPatient(m.patient);
    const phone=phoneText(p)||'Sin teléfono registrado';
    dialog('Revisar contacto de '+esc(p.name),`<p>Teléfono registrado: <span class="mono">${esc(phone)}</span></p><p>Canal preferido: ${esc(p.contact||'No definida')}</p><p class="small muted">Comprueba el número en la ficha antes de cualquier otro intento de envío. Este aviso no reenvía el mensaje.</p>`,button('Volver al mensaje',`message:${m.id}`,'primary')+link('Ver ficha del paciente',`paciente/${p.id}`,'button'));
  }
  else if(action==='comm-filters')dialog('Filtrar comunicaciones',`<div class="fields"><label>Estado<select id="comm-status"><option value="">Todos los estados</option>${options(['Programado','Entregado','Respondido','Revisar teléfono','Resultado desconocido'],state.commFilter)}</select></label></div><p class="field-help">El filtro no cambia los mensajes ni sus resultados; solo acorta la lista.</p>`,button('Aplicar filtro','apply-comm-filter','primary'));
  else if(action==='apply-comm-filter'){state.commFilter=document.getElementById('comm-status').value;closeDialog();render();}
  else if(action==='clear-comm-filter'){state.commFilter='';render();}
  else if(action==='dismiss-review'){if(!state.reviewDismissed.includes(id))state.reviewDismissed.push(id);render();toast('Asunto marcado como revisado en el ejemplo.');}
  else if(action==='metric-filters')dialog('Elegir periodo del informe',`<div class="fields"><label>Periodo<select id="metric-period">${metricPeriods.map(p=>`<option value="${p.id}" ${p.id===state.metricPeriod?'selected':''}>${p.label}</option>`).join('')}</select></label></div><p class="field-help">Los periodos en curso muestran los datos disponibles hasta el 21 de octubre y pueden crecer al cerrarse.</p>`,button('Ver informe','apply-metric-filter','primary'));
  else if(action==='apply-metric-filter'){state.metricPeriod=document.getElementById('metric-period').value;closeDialog();render();}
  else if(action==='metric-period'){state.metricPeriod=id;render();}
  else if(action==='metrics-retry'){if(state.scenario==='unavailable')toast('El servicio sigue sin estar disponible en el ejemplo.');else render();}
  else if(action==='add-service'||action==='edit-service'){
    const c=id?serviceCatalog.find(s=>s.id===id):null;
    const o=c?offerByServiceId(c.id):null;
    dialog(c?'Editar servicio':'Añadir servicio',`<form id="service-form" data-service="${c?c.id:''}"><div class="fields"><label>Nombre<input name="name" value="${c?esc(c.name):''}"></label><div class="two-fields"><label>Duración base del catálogo · minutos<input name="baseDuration" type="number" value="${c?c.baseDuration:30}"></label><label>Ajuste de esta consulta · minutos<input name="override" type="number" value="${o&&o.override?o.override:''}" placeholder="Sin ajuste"></label></div><p class="field-help">La duración de referencia usa el ajuste de la oferta cuando existe y, si no, la duración base del catálogo. Las citas existentes conservan su duración.</p><label class="check-line"><input name="active" type="checkbox" ${!o||o.active?'checked':''}> Oferta activa en esta consulta</label></div><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button class="primary" type="submit">${c?'Guardar cambios':'Añadir servicio'}</button></div></form>`);
  }
  else if(action==='toggle-offer'){
    const o=serviceOffers.find(o=>o.id===id);
    if(!o)return;
    if(state.viewerRole!=='owner'){toast('Solo la persona que administra la consulta puede cambiar la oferta.');return;}
    const c=serviceCatalog.find(s=>s.id===o.serviceId);
    if(o.active)dialog('¿Desactivar la oferta de '+esc(c.name)+'?','<p>Dejará de aparecer al crear citas nuevas. Las citas existentes conservan su servicio y su duración.</p>',button('Desactivar oferta',`do-toggle-offer:${id}`,'primary'));
    else{o.active=true;render();toast('Oferta activada en el ejemplo.');}
  }
  else if(action==='do-toggle-offer'){const o=serviceOffers.find(o=>o.id===id);o.active=false;closeDialog();render();toast('Oferta desactivada en el ejemplo. Las citas existentes conservan sus datos.');}
  else if(action==='toggle-catalog'){
    const c=serviceCatalog.find(s=>s.id===id);
    if(!c)return;
    if(state.viewerRole!=='owner'){toast('Solo la persona que administra la consulta puede cambiar el catálogo.');return;}
    if(c.active!==false)dialog('¿Inactivar '+esc(c.name)+' en el catálogo?','<p>El servicio dejará de ofrecerse en esta consulta y su oferta quedará inactiva. Las citas existentes conservan su servicio y su duración.</p>',button('Inactivar en catálogo',`do-toggle-catalog:${id}`,'primary'));
    else{c.active=true;render();toast('Servicio activo en el catálogo.');}
  }
  else if(action==='do-toggle-catalog'){const c=serviceCatalog.find(s=>s.id===id);c.active=false;const o=offerByServiceId(id);if(o)o.active=false;closeDialog();render();toast('Servicio inactivado en el ejemplo. Las citas existentes conservan sus datos.');}
  else if(action==='add-rule')dialog('Añadir regla semanal',ruleForm());
  else if(action==='edit-rule')dialog('Editar regla semanal',ruleForm(scheduleRules.find(r=>r.id===id)));
  else if(action==='add-exception')dialog('Añadir excepción',exceptionForm());
  else if(action==='edit-exception')dialog('Editar excepción',exceptionForm(scheduleExceptions.find(e=>e.id===id)));
  else if(action==='add-block')dialog('Añadir bloqueo',blockForm());
  else if(action==='edit-block')dialog('Editar bloqueo',blockForm(scheduleBlocks.find(b=>b.id===id)));
  else if(action==='add-interval-row')document.getElementById('interval-rows').insertAdjacentHTML('beforeend',intervalRows([{start:'09:00',end:'12:00'}]));
  else if(action==='add-ex-interval-row')document.getElementById('ex-interval-rows').insertAdjacentHTML('beforeend',intervalRows([{start:'09:00',end:'12:00'}]));
  else if(action==='invite')dialog('Invitar a un asistente',`<form id="invite-form"><div class="fields"><label>Correo de la persona<input name="email" placeholder="asistente@example.test"></label><label>Rol<select name="role"><option>Asistente</option></select></label></div><p class="field-help">Solo existen los roles definidos; no hay roles personalizados. El envío no se ejecuta: se registra la invitación en el ejemplo.</p><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Registrar invitación de ejemplo</button></div></form>`);
  else if(action==='reinvite'){
    const i=invitations.find(v=>v.id===id);
    if(!i)return;
    dialog('Invitar a un asistente',`<form id="invite-form" data-reinvite="${i.id}"><div class="fields"><label>Correo de la persona<input name="email" value="${esc(i.email)}"></label><label>Rol<select name="role"><option>Asistente</option></select></label></div><p class="field-help">La invitación anterior venció. Registrar una nueva no envía ningún correo real.</p><div id="dialog-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Registrar invitación de ejemplo</button></div></form>`);
  }
  else if(action==='revoke-invite'){
    const i=invitations.find(v=>v.id===id);
    if(!i)return;
    dialog('¿Retirar la invitación para '+esc(i.email)+'?','<p>El enlace dejará de servir y la persona no podrá unirse con esa invitación. Puedes invitarla de nuevo cuando quieras.</p><div id="dialog-error" class="error" role="alert"></div>',button('Retirar invitación',`do-revoke-invite:${i.id}`,'primary'));
  }
  else if(action==='do-revoke-invite'){
    const i=invitations.find(v=>v.id===id);
    if(!i)return;
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Retirar la invitación requiere conexión y autorización. Tus cambios se conservan.');
    invitations.splice(invitations.indexOf(i),1);
    closeDialog();render();toast('Invitación retirada en el ejemplo.');
  }
  else if(action==='person'){
    const m=teamMembers.find(m=>m.id===id);
    if(!m)return;
    const owner=state.viewerRole==='owner';
    dialog('Acceso de '+esc(m.name),`<p>${esc(m.role)} · ${m.active?'Acceso activo':'Acceso retirado'}</p><p class="muted">${esc(m.access)}</p><p class="small muted">Correo: ${esc(m.email)}</p><div id="dialog-error" class="error" role="alert"></div><p class="small subtle">Los cambios requieren conexión y autorización; los equipos desconectados reciben el cambio al reconectarse.</p>`,m.active?(owner?button('Retirar acceso',`remove-access:${m.id}`,'primary'):'<p class="small subtle">Solo la persona que administra la consulta puede retirar el acceso.</p>'):'');
  }
  else if(action==='remove-access'){
    const m=teamMembers.find(m=>m.id===id);
    if(!m)return;
    dialog('¿Retirar el acceso de '+esc(m.name)+'?','<p>Perderá acceso a la consulta: agenda, pacientes y configuración. Sus datos registrados se conservan.</p><p>Los equipos desconectados no reciben el cambio de inmediato: se aplicará cuando vuelvan a conectarse.</p><div id="dialog-error" class="error" role="alert"></div>',button('Retirar acceso',`do-remove-access:${m.id}`,'primary'));
  }
  else if(action==='do-remove-access'){
    const m=teamMembers.find(m=>m.id===id);
    if(!m)return;
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Retirar el acceso requiere conexión y autorización. La persona conserva su acceso hasta entonces.');
    m.active=false;
    closeDialog();render();toast('Acceso retirado en el ejemplo; se aplicará a los equipos al conectarse.');
  }
  else if(action==='device'){
    const d=devices.find(v=>v.id===id);
    if(!d)return;
    const owner=state.viewerRole==='owner';
    const statusText=d.status==='current'?'Este equipo':d.status==='revoking'?'Revocación registrada; falta confirmarla con el servicio':d.status==='revoked'?'Acceso restringido':'Autorizado';
    const revoke=d.status==='active'?(owner?button('Revocar acceso',`revoke-device:${d.id}`,'primary'):''):'';
    dialog('Acceso del equipo '+esc(d.name),`<p>${esc(d.os)} · ${esc(d.app)}</p><p>Último acceso conocido: ${esc(d.last)}</p><p>Autorizado: ${esc(d.authorized)}</p><p>Estado: ${statusText}</p><div id="dialog-error" class="error" role="alert"></div><p class="small subtle">No se muestran secretos de sesión. Los equipos desconectados no reciben cambios de inmediato.</p>`,revoke);
  }
  else if(action==='revoke-device'){
    const d=devices.find(v=>v.id===id);
    if(!d)return;
    dialog('¿Revocar el acceso de '+esc(d.name)+'?','<p>El equipo dejará de sincronizar y deberá volver a autorizarse. Los datos locales del equipo no se borran desde aquí.</p><p>El cambio se aplica cuando el equipo esté conectado; no es inmediato si está desconectado.</p><div id="dialog-error" class="error" role="alert"></div>',button('Revocar acceso',`do-revoke-device:${d.id}`,'primary'));
  }
  else if(action==='do-revoke-device'){
    const d=devices.find(v=>v.id===id);
    if(!d)return;
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Revocar el acceso requiere conexión y autorización. Tus cambios se conservan.');
    d.status='revoking';
    closeDialog();render();toast('Revocación registrada en el ejemplo; falta confirmarla con el servicio.');
  }
  else if(action==='device-recheck'){
    const d=devices.find(v=>v.id===id);
    if(!d)return;
    if(['offline','unavailable'].includes(state.scenario))return toast('Seguimos sin poder confirmar la revocación en el ejemplo.');
    d.status='revoked';render();toast('Revocación confirmada en el ejemplo; el equipo queda restringido.');
  }
  else if(action==='device-reactivate'){
    const d=devices.find(v=>v.id===id);
    if(!d)return;
    dialog('¿Autorizar de nuevo '+esc(d.name)+'?','<p>El equipo volverá a sincronizar con la consulta. La autorización queda registrada en el ejemplo.</p>',button('Autorizar equipo',`do-device-reactivate:${d.id}`,'primary'));
  }
  else if(action==='do-device-reactivate'){
    const d=devices.find(v=>v.id===id);
    if(!d)return;
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Autorizar de nuevo requiere conexión y autorización. Tus cambios se conservan.');
    d.status='active';d.last='Ahora';
    closeDialog();render();toast('Equipo autorizado de nuevo en el ejemplo.');
  }
  else if(action==='discard-conflict')dialog('¿Descartar tu propuesta?','<p>El horario aceptado en la consulta se mantiene. Solo se retira tu propuesta pendiente en este ejemplo.</p>',button('Descartar propuesta','do-discard-conflict','primary'));
  else if(action==='do-discard-conflict'){state.scenario='normal';closeDialog();go('agenda');toast('Propuesta descartada en el ejemplo.');}
  else if(action==='hub-confirm'){state.hubStatus='Confirmada';go(state.scenario==='unavailable'?'hub/no-disponible':'hub/confirmada');}
  else if(action==='hub-cancel'){state.hubStatus='Cancelada';go(state.scenario==='unavailable'?'hub/no-disponible':'hub/cancelada');}
  else if(action==='hub-service-retry')toast('El servicio sigue sin estar disponible en el ejemplo.');
  else if(action==='hub-retry')toast('Ejemplo: el resultado sigue pendiente de comprobar.');
  else if(action==='scenarios')dialog('Probar un escenario',`<p>Estos controles pertenecen al prototipo, no a la aplicación.</p><div class="fields"><label>Vista de usuario<select id="prototype-viewer"><option value="owner-one" ${state.viewerRole==='owner'&&state.professionalCount===1?'selected':''}>Dueña · un profesional</option><option value="assistant-one" ${state.viewerRole==='assistant'?'selected':''}>Asistente · un profesional</option><option value="owner-many" ${state.professionalCount>1?'selected':''}>Dueña · varios profesionales</option></select></label><label>Permiso de citas<select id="prototype-appointment-access"><option value="edit" ${state.canEditAppointments?'selected':''}>Consultar y gestionar</option><option value="read" ${!state.canEditAppointments?'selected':''}>Solo lectura</option></select></label><label>Hora del ejemplo · Caracas<input type="datetime-local" id="prototype-now" value="${state.exampleNow}"></label><p class="field-help">Margen de ausencia del ejemplo: 15 minutos, según el contrato vigente.</p><label>Estado de Agenda<select id="agenda-mode">${[['ready','Datos disponibles'],['loading','Carga inicial'],['refreshing','Actualización con datos visibles'],['stale','Datos desactualizados'],['read-error','Error de lectura local'],['no-permission','Sin permiso de consulta']].map(([value,label])=>`<option value="${value}" ${state.agendaMode===value?'selected':''}>${label}</option>`).join('')}</select></label><fieldset class="scenario-options"><legend>Estado del ejemplo</legend>${[['normal','Funcionamiento normal'],['offline','Sin conexión · trabajo local permitido'],['unavailable','Servicio temporalmente no disponible'],['conflict','Conflicto de horario']].map(([value,label])=>`<label class="radio-line"><input type="radio" name="scenario" value="${value}" ${state.scenario===value?'checked':''}>${label}</label>`).join('')}</fieldset></div>`,button('Mostrar escenario','apply-scenario','primary'));

  else if(action==='apply-scenario'){state.scenario=document.querySelector('input[name=scenario]:checked').value;state.agendaMode=document.getElementById('agenda-mode').value;state.canEditAppointments=document.getElementById('prototype-appointment-access').value==='edit';state.exampleNow=document.getElementById('prototype-now').value||state.exampleNow;const viewer=document.getElementById('prototype-viewer').value;state.viewerRole=viewer==='assistant-one'?'assistant':'owner';state.professionalCount=viewer==='owner-many'?2:1;if(!showProfessional()||!professionals.slice(0,state.professionalCount).some(p=>p.id===state.professionalFilter))state.professionalFilter='';if(state.scenario==='normal'&&canEditAppointments())for(const a of state.appointments.filter(a=>a.pendingAction)){a.history=[...a.history,...a.localActivity.slice().reverse()];a.localActivity=[];a.accepted=appointmentSnapshot(a);a.pendingAction=false;}closeDialog();render();}
  else if(action==='reset')dialog('Restablecer los datos de ejemplo','<p>Se retirarán las ediciones hechas durante esta visita al prototipo.</p>',button('Restablecer','do-reset','primary'));
  else if(action==='do-reset'){location.reload();}
  else if(action==='keep-editing'){pendingNavigation=null;closeDialog();}
  else if(action==='discard-edit'){const target=pendingNavigation;pendingNavigation=null;activeDraft=null;document.querySelector('form[data-dirty]')?.removeAttribute('data-dirty');closeDialog();location.hash=target;}
});

document.addEventListener('input',event=>{
  const el=event.target;
  if(el.matches('.rule-start,.rule-end')){const row=el.closest('.interval-row');row.querySelector('.interval-duration').textContent=intervalDuration(row.querySelector('.rule-start').value,row.querySelector('.rule-end').value);}
  if(el.closest('#appointment-form')){
    const form=el.closest('form');form.dataset.dirty='true';
    const original=state.appointments.find(a=>a.id===form.dataset.id);
    if(original&&el.name==='start'){const end=minutes(el.value)+minutes(original.end)-minutes(original.start);form.elements.end.value=end<1440?clock(end):'';}
    const draft={patient:original?.patient,professionalId:original?.professionalId,service:original?.service,responsibleContactId:original?.responsibleContactId,...formValues(form)};
    activeDraft=draft;
    if(el.id==='booking-patient-search'){draft.patient='';document.getElementById('booking-patient').innerHTML=bookingPatientOptions(draft);document.getElementById('booking-search-note').textContent=document.getElementById('booking-patient').options.length===1?'Sin coincidencias en los datos locales. Puedes registrar una ficha.':'Elige el paciente y revisa su contacto.';}
    document.getElementById('appointment-summary').innerHTML=editorSummary(draft,original);
  }
  if(el.closest('#inline-patient-form'))state.inlinePatientDraft=formValues(el.closest('form'));
  if(el.closest('#patient-form'))el.closest('form').dataset.dirty='true';
  if(el.closest('#setup-form'))el.closest('form').dataset.dirty='true';
  if(el.id==='global-search'){state.lastSearch=el.value;document.getElementById('global-results').innerHTML=searchResults(el.value);}
  if(['patient-search','appointment-search'].includes(el.id)){const selection=el.selectionStart;if(el.id==='patient-search')state.query=el.value;else state.appointmentQuery=el.value;state.listLimit=8;render();const replacement=document.getElementById(el.id);replacement.focus();replacement.setSelectionRange(selection,selection);}
});
document.addEventListener('change',event=>{
  if(event.target.name==='identificationBasis')event.target.closest('form').querySelectorAll('[data-basis]').forEach(group=>{group.hidden=group.dataset.basis!==event.target.value;});
  if(event.target.name==='exType'){const panel=event.target.closest('form').querySelector('[data-ex-intervals]');if(panel)panel.hidden=event.target.value==='closed';}
  if(event.target.closest('#appointment-form')&&event.target.id!=='booking-patient-search'){activeDraft=formValues(event.target.closest('form'));if(event.target.id==='booking-patient')activeDraft.responsibleContactId='';if(event.target.id==='booking-service'&&!event.target.closest('form').dataset.id)activeDraft.end=clock(minutes(activeDraft.start)+referenceDuration(activeDraft.service));render();}
  if(event.target.id==='agenda-date'){state.date=event.target.value||TODAY;render();}
  if(event.target.id==='appointment-filter'){state.filter=event.target.value;render();}
  if(event.target.id==='list-period'){state.listPeriod=event.target.value;state.listLimit=8;render();}
  if(event.target.id==='professional-filter'){state.professionalFilter=event.target.value;render();}
});
document.addEventListener('submit',event=>{
  const form=event.target;event.preventDefault();
  const values=formValues(form);
  if(form.id==='setup-form'){
    if(state.viewerRole!=='owner')return errorMessage('Solo la persona que administra la consulta puede completar estos datos.');
    if(['offline','unavailable'].includes(state.scenario))return errorMessage('Necesitas conexión y acceso al servicio para terminar la preparación. Tus datos se conservan.');
    if(!values.practice.trim()||!values.service.trim()||!values.location.trim())return errorMessage('Completa el nombre, el lugar y el servicio de la consulta.');
    if(values.end<=values.start)return errorMessage('La hora de fin debe ser posterior a la de inicio.');
    state.setup={...values,reviewed:true};
    ensureService(values.service.trim());
    form.removeAttribute('data-dirty');go('preparacion/lista');
  }else if(form.id==='appointment-form'){
    saveAppointment(form,values);
  }else if(form.id==='inline-patient-form'){
    if(form.dataset.submitted)return;
    if(!values.name.trim()||!values.documentNumber.trim()||!values.phone.trim()||!values.phoneCode.trim()){document.getElementById('inline-patient-error').textContent='Completa nombre, documento y teléfono con su código. Tus campos se conservan.';return;}
    form.dataset.submitted='true';
    const p={id:'p'+(state.patients.length+1),name:values.name.trim(),phoneCode:values.phoneCode.trim(),phoneNumber:values.phone.trim(),email:'',contact:'No definida',visits:0,active:true,completion:'complete',identificationBasis:'own_document',identifier:{country:values.country,type:values.documentType,number:values.documentNumber.trim(),status:'declared'}};
    state.patients.push(p);
    activeDraft.patient=p.id;activeDraft.patientSearch='';activeDraft.responsibleContactId='';state.inlinePatientDraft=null;closeDialog();render();document.getElementById('appointment-form').dataset.dirty='true';document.getElementById('booking-patient').focus();
    toast('Paciente registrado en el ejemplo.');
  }else if(form.id==='patient-form'){
    if(form.dataset.submitted)return;
    const draft=form.dataset.saveAs==='draft';
    form.dataset.saveAs='complete';
    const basis=values.identificationBasis,name=values.name.trim();
    const pairComplete=!!(values.phoneCode.trim()&&values.phoneNumber.trim()),pairEmpty=!values.phoneCode.trim()&&!values.phoneNumber.trim();
    const fail=msg=>errorMessage(msg);
    if(!name)return fail('Escribe el nombre del paciente.');
    if(!draft&&!pairComplete&&!pairEmpty)return fail('Completa el código y el número de teléfono juntos, o deja ambos vacíos.');
    if(!draft&&basis==='own_document'&&!values.identifierNumber.trim())return fail('Completa el número de documento o guarda un borrador.');
    if(!draft&&basis==='minor_without_document'&&!values.birthDate)return fail('Indica la fecha de nacimiento o guarda un borrador.');
    if(!draft&&basis==='minor_without_document'&&!values.responsibleId)return fail('Elige un responsable vigente con teléfono.');
    if(!draft&&basis==='own_document'&&!pairComplete&&!values.responsibleId)return fail('Añade un teléfono completo o elige un responsable vigente.');
    form.dataset.submitted='true';
    const p={id:'p'+(state.patients.length+1),name,phoneCode:pairEmpty?'':values.phoneCode.trim(),phoneNumber:pairEmpty?'':values.phoneNumber.trim(),email:values.email||'',contact:values.contact?'WhatsApp':'No definida',visits:0,active:true,completion:draft?'draft':'complete',identificationBasis:basis,birthDate:basis==='minor_without_document'?values.birthDate:'',needsResponsible:basis==='minor_without_document',responsibleIds:values.responsibleId?[values.responsibleId]:[],identifier:basis==='own_document'&&values.identifierNumber.trim()?{country:values.identifierCountry,type:values.identifierType,number:values.identifierNumber.trim(),status:'declared'}:null};
    state.patients.push(p);
    form.removeAttribute('data-dirty');go('paciente/'+p.id);
    toast(draft?'Borrador guardado en el ejemplo.':'Paciente registrado en el ejemplo.');
  }else if(form.id==='edit-patient-form'){
    const p=getPatient(form.dataset.id),basis=values.identificationBasis,name=values.name.trim();
    const pairComplete=!!(values.phoneCode.trim()&&values.phoneNumber.trim()),pairEmpty=!values.phoneCode.trim()&&!values.phoneNumber.trim();
    const meetsComplete=()=>name&&(basis==='own_document'?!!(values.identifierNumber.trim()&&(pairComplete||values.responsibleId)):!!(values.birthDate&&values.responsibleId));
    if(!name)return dialogError('Escribe el nombre del paciente.');
    if(!pairComplete&&!pairEmpty)return dialogError('Completa el código y el número de teléfono juntos, o deja ambos vacíos.');
    if(p.completion==='complete'&&!meetsComplete())return dialogError('Estos cambios dejarían la ficha completa sin sus requisitos. Corrige el documento y añade un teléfono completo o un responsable; tu edición se conserva.');
    Object.assign(p,{name,phoneCode:pairEmpty?'':values.phoneCode.trim(),phoneNumber:pairEmpty?'':values.phoneNumber.trim(),email:values.email||'',contact:values.contact?'WhatsApp':'No definida',identificationBasis:basis,birthDate:basis==='minor_without_document'?values.birthDate:'',needsResponsible:basis==='minor_without_document',responsibleIds:values.responsibleId?[values.responsibleId]:[],identifier:basis==='own_document'&&values.identifierNumber.trim()?{...(p.identifier||{}),country:values.identifierCountry,type:values.identifierType,number:values.identifierNumber.trim(),status:p.identifier?.status||'declared'}:null});
    if(p.completion==='draft'&&meetsComplete()){p.completion='complete';toast('Ficha completa en el ejemplo.');}
    else toast('Información actualizada en el ejemplo.');
    closeDialog();render();
  }
  else if(form.id==='responsible-form'){
    const cid=form.dataset.contact,pid=form.dataset.patient;
    const name=values.name.trim(),rel=values.relationship.trim();
    const pairComplete=!!(values.phoneCode.trim()&&values.phoneNumber.trim());
    if(!name||!rel)return dialogError('Completa el nombre y el parentesco. Tus campos se conservan.');
    if(!pairComplete)return dialogError('El responsable necesita un teléfono completo: código y número juntos. Tus campos se conservan.');
    if(cid){
      Object.assign(responsibleContacts.find(c=>c.id===cid),{name,relationship:rel,role:values.role,phoneCode:values.phoneCode.trim(),phoneNumber:values.phoneNumber.trim(),representation:values.representation});
      toast('Responsable actualizado en el ejemplo.');
    }else{
      const c={id:'c'+(responsibleContacts.length+1),name,relationship:rel,role:values.role,phoneCode:values.phoneCode.trim(),phoneNumber:values.phoneNumber.trim(),representation:values.representation,active:true,since:TODAY};
      responsibleContacts.push(c);
      const patient=getPatient(pid);
      patient.responsibleIds=[...(patient.responsibleIds||[]),c.id];
      toast('Responsable añadido en el ejemplo.');
    }
    closeDialog();render();
  }else if(form.id==='inline-responsible-form'){
    if(form.dataset.submitted)return;
    if(!values.name.trim()||!values.relationship.trim()||!values.phoneCode.trim()||!values.phoneNumber.trim()){document.getElementById('inline-responsible-error').textContent='Completa nombre, parentesco y teléfono con su código. Tus campos se conservan.';return;}
    form.dataset.submitted='true';
    const c={id:'c'+(responsibleContacts.length+1),name:values.name.trim(),relationship:values.relationship.trim(),role:values.role||'Contacto y decisiones administrativas',phoneCode:values.phoneCode.trim(),phoneNumber:values.phoneNumber.trim(),representation:values.representation||'declared',active:true,since:TODAY};
    responsibleContacts.push(c);
    const patient=state.patients.find(p=>p.id===form.dataset.patient);
    if(patient)patient.responsibleIds=[...(patient.responsibleIds||[]),c.id];
    if(activeDraft)activeDraft.responsibleContactId=c.id;
    closeDialog();render();
    document.getElementById('booking-responsible')?.focus();
    toast('Responsable registrado en el ejemplo.');
  }
  else if(form.id==='waitlist-form'){
    if(!values.preference||!values.preference.trim())return dialogError('Escribe la preferencia de horario. Tus cambios se conservan.');
    const entryId=form.dataset.waitlist,w=entryId?state.waitlist.find(w=>w.id===entryId):null;
    if(w){w.service=values.service;w.preference=values.preference.trim();}
    else state.waitlist.push({id:'w'+(state.waitlist.length+1),...values,preference:values.preference.trim(),since:TODAY});
    closeDialog();render();toast(w?'Entrada actualizada en el ejemplo.':'Paciente añadido a la lista de espera.');
  }
  else if(form.id==='hub-request-form')go('hub/solicitud');
  else if(form.id==='account-form'){
    if(state.viewerRole!=='owner')return errorMessage('Solo la persona que administra la consulta puede cambiar estos datos.');
    if(['offline','unavailable'].includes(state.scenario))return errorMessage('Cambiar los datos compartidos requiere conexión y acceso al servicio. Tus cambios se conservan en el formulario.');
    const place=values.placeName.trim(),city=values.city.trim(),region=values.state.trim();
    if(!values.practice.trim()||!values.professional.trim())return errorMessage('Completa el nombre de la consulta y de la persona profesional. Tus cambios se conservan.');
    if(!place||!city||!region)return errorMessage('La dirección de lectura debe llegar al menos a ciudad y estado. Revisa los campos; tus cambios se conservan.');
    state.account={practice:values.practice.trim(),professional:values.professional.trim(),specialty:values.specialty.trim(),place:{name:place,room:values.placeRoom.trim(),street:values.placeStreet.trim(),city,state:region}};
    render();
    toast('Cambios guardados en el ejemplo. Las citas existentes conservan sus datos.');
  }
  else if(form.id==='local-form'){
    state.localPrefs={hourFormat:values.hourFormat};
    toast('Preferencias locales guardadas en el ejemplo.');
  }
  else if(form.id==='settings-form'){
    if(state.viewerRole!=='owner')return errorMessage('Solo la persona que administra la consulta puede cambiar la política de comunicaciones. Tus cambios se conservan.');
    if(['offline','unavailable'].includes(state.scenario))return errorMessage('Cambiar la configuración compartida requiere conexión y acceso al servicio. Tus cambios se conservan.');
    render();
    toast('Configuración guardada en el ejemplo. No se envían mensajes por este cambio.');
  }
  else if(form.id==='service-form'){
    const cid=form.dataset.service;
    const name=values.name.trim(),base=Number(values.baseDuration),override=values.override===''?null:Number(values.override);
    if(state.viewerRole!=='owner')return dialogError('Solo la persona que administra la consulta puede cambiar el catálogo y la oferta.');
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Cambiar el catálogo o la oferta requiere conexión y acceso al servicio. Tus cambios se conservan.');
    if(!name)return dialogError('Escribe el nombre del servicio. Tus cambios se conservan.');
    if(!Number.isInteger(base)||base<1||base>1440)return dialogError('La duración base debe ser un número entero entre 1 y 1440 minutos. Tus cambios se conservan.');
    if(override!==null&&(!Number.isInteger(override)||override<1||override>1440))return dialogError('El ajuste debe ser un entero entre 1 y 1440 minutos, o dejarse vacío. Tus cambios se conservan.');
    if(serviceCatalog.some(s=>s.name===name&&s.id!==cid))return dialogError('Ya existe un servicio con ese nombre en el catálogo. Tus cambios se conservan.');
    if(cid){
      const c=serviceCatalog.find(s=>s.id===cid);
      Object.assign(c,{name,baseDuration:base});
      Object.assign(offerByServiceId(cid),{override,active:!!values.active});
      toast('Servicio y oferta actualizados en el ejemplo. Las citas existentes conservan sus datos.');
    }else{
      const nid='s'+(serviceCatalog.length+1);
      serviceCatalog.push({id:nid,name,baseDuration:base,active:true});
      serviceOffers.push({id:'o_'+nid,serviceId:nid,override,active:!!values.active});
      toast('Servicio añadido al catálogo en el ejemplo.');
    }
    closeDialog();render();
  }
  else if(form.id==='rule-form'){
    const rid=form.dataset.rule;
    const days=Array.from(form.querySelectorAll('input[name=day]:checked')).map(i=>Number(i.value));
    const intervals=Array.from(form.querySelectorAll('#interval-rows .interval-row')).map(row=>({start:row.querySelector('.rule-start').value,end:row.querySelector('.rule-end').value}));
    if(state.viewerRole!=='owner')return dialogError('Solo la persona profesional puede cambiar el horario.');
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Cambiar el horario requiere conexión y acceso al servicio. Tus cambios se conservan.');
    if(!days.length)return dialogError('Marca al menos un día de la semana. Tus cambios se conservan.');
    if(!values.validFrom||!values.validTo||values.validTo<values.validFrom)return dialogError('Revisa la vigencia: el inicio debe ser anterior o igual al fin. Tus cambios se conservan.');
    if(!intervals.length||intervals.some(i=>!i.start||!i.end||i.end<=i.start))return dialogError('Cada tramo necesita una hora de inicio anterior a la de fin. Tus cambios se conservan.');
    const snapshotRules=structuredClone(scheduleRules);
    const original=rid?scheduleRules.find(r=>r.id===rid):null;
    if(original)Object.assign(original,{days,intervals,validFrom:values.validFrom,validTo:values.validTo});
    else scheduleRules.push({id:'r'+(scheduleRules.length+1),days,intervals,validFrom:values.validFrom,validTo:values.validTo});
    const affected=state.appointments.filter(a=>a.professionalId==='elena'&&a.status!=='Cancelada'&&!terminalStatuses.includes(a.status)&&a.date>=values.validFrom&&a.date<=values.validTo&&days.includes(new Date(a.date+'T12:00:00Z').getUTCDay())&&!windowsForDate(a.date).some(w=>minutes(a.start)>=w.start&&minutes(a.end)<=w.end));
    if(affected.length){scheduleRules.length=0;scheduleRules.push(...snapshotRules);return dialogError(`Este cambio dejaría ${affected.length} cita(s) vigente(s) fuera del horario: ${affected.map(a=>`${getPatient(a.patient).name} (${fmt(a.date)} ${a.start})`).join('; ')}. Reprograma esas citas o ajusta los tramos. Tus cambios se conservan.`);}
    closeDialog();render();toast('Horario guardado en el ejemplo.');
  }
  else if(form.id==='exception-form'){
    const eid=form.dataset.exception;
    const exType=values.exType;
    const intervals=Array.from(form.querySelectorAll('#ex-interval-rows .interval-row')).map(row=>({start:row.querySelector('.rule-start').value,end:row.querySelector('.rule-end').value}));
    if(state.viewerRole!=='owner')return dialogError('Solo la persona profesional puede cambiar las excepciones.');
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Cambiar las excepciones requiere conexión y acceso al servicio. Tus cambios se conservan.');
    if(!values.date)return dialogError('Elige la fecha de la excepción. Tus cambios se conservan.');
    if(exType==='replacement'&&( !intervals.length||intervals.some(i=>!i.start||!i.end||i.end<=i.start)))return dialogError('Un reemplazo necesita al menos un tramo con inicio anterior a fin. Tus cambios se conservan.');
    const snapshotEx=structuredClone(scheduleExceptions);
    const original=eid?scheduleExceptions.find(e=>e.id===eid):null;
    const newEx={id:eid||'e'+(scheduleExceptions.length+1),date:values.date,type:exType,intervals:exType==='replacement'?intervals:[],label:values.label||''};
    if(original)Object.assign(original,newEx);else scheduleExceptions.push(newEx);
    const affected=state.appointments.filter(a=>a.professionalId==='elena'&&a.date===values.date&&a.status!=='Cancelada'&&!terminalStatuses.includes(a.status)&&!windowsForDate(values.date).some(w=>minutes(a.start)>=w.start&&minutes(a.end)<=w.end));
    if(affected.length){scheduleExceptions.length=0;scheduleExceptions.push(...snapshotEx);return dialogError(`Esta excepción dejaría ${affected.length} cita(s) vigente(s) fuera del horario: ${affected.map(a=>`${getPatient(a.patient).name} ${a.start}`).join('; ')}. Reprograma esas citas o cambia la excepción. Tus cambios se conservan.`);}
    closeDialog();render();toast('Excepción guardada en el ejemplo.');
  }
  else if(form.id==='block-form'){
    const bid=form.dataset.block;
    if(state.viewerRole!=='owner')return dialogError('Solo la persona profesional puede gestionar bloqueos.');
    if(['offline','unavailable'].includes(state.scenario))return dialogError('Crear o cambiar bloqueos requiere conexión y acceso al servicio. Tus cambios se conservan.');
    if(values.scope==='professional'&&state.viewerRole!=='owner')return dialogError('Solo la profesional puede bloquear toda su actividad.');
    if(!values.date||!values.start||!values.end||values.end<=values.start)return dialogError('Revisa la fecha y el intervalo del bloqueo: el fin debe ser posterior al inicio. Tus cambios se conservan.');
    if(!values.label.trim())return dialogError('Escribe un motivo para el bloqueo. Tus cambios se conservan.');
    const affected=state.appointments.filter(a=>a.professionalId==='elena'&&a.date===values.date&&a.status!=='Cancelada'&&!terminalStatuses.includes(a.status)&&a.start<values.end&&a.end>values.start);
    if(affected.length)return dialogError(`Este bloqueo cruza ${affected.length} cita(s) vigente(s): ${affected.map(a=>`${getPatient(a.patient).name} ${a.start} – ${a.end}`).join('; ')}. Reprograma esas citas o elige otro intervalo. Tus cambios se conservan.`);
    const entry={id:bid||'b'+(scheduleBlocks.length+1),date:values.date,start:values.start,end:values.end,professionalId:'elena',global:values.scope==='professional',label:values.label.trim()};
    const original=bid?scheduleBlocks.find(b=>b.id===bid):null;
    if(original)Object.assign(original,entry);else scheduleBlocks.push(entry);
    closeDialog();render();toast('Bloqueo guardado en el ejemplo.');
  }
  else if(form.id==='invite-form'){
    const email=(values.email||'').trim();
    if(!email||!email.includes('@'))return dialogError('Escribe el correo de la persona. Tus cambios se conservan.');
    if(invitations.some(i=>i.email===email&&i.status==='pending'))return dialogError('Ya hay una invitación pendiente para ese correo. Tus cambios se conservan.');
    const reinviteId=form.dataset.reinvite;
    if(reinviteId){const old=invitations.find(i=>i.id===reinviteId);if(old)invitations.splice(invitations.indexOf(old),1);}
    invitations.push({id:'inv'+(invitations.length+1),email,role:values.role||'Asistente',sent:TODAY,status:'pending'});
    closeDialog();render();toast('Invitación registrada en el ejemplo; no se envía ningún correo real.');
  }
});
document.addEventListener('click',event=>{
  if(event.target.closest('.skip')){event.preventDefault();const main=document.getElementById('main');main.setAttribute('tabindex','-1');main.focus();return;}
  const anchor=event.target.closest('a[href^="#/"]');
  if(!anchor)return;
  if(anchor.hash.startsWith('#/cita/')&&!route.startsWith('cita/')){
    state.detailOrigin=route;
    const scroll=document.querySelector('.week-scroll');
    state.agendaScroll={top:window.scrollY,left:scroll?.scrollLeft||0,weekTop:scroll?.scrollTop||0};
  }
  if(document.getElementById('dialog').open){if(document.getElementById('dialog').querySelector('form[data-dirty]'))return;closeDialog();}
  const dirty=document.querySelector('form[data-dirty]');
  if(dirty&&anchor.hash!==location.hash){event.preventDefault();pendingNavigation=anchor.hash;dialog('Hay cambios sin guardar','<p>Puedes seguir editando o descartar los cambios del formulario.</p>',button('Seguir editando','keep-editing')+button('Descartar cambios','discard-edit','primary'));return;}
  if(!anchor.hash.includes('cita/nueva'))activeDraft=null;
});
window.addEventListener('hashchange',()=>{
  const from=route,el=document.getElementById('dialog');
  if(document.querySelector('form[data-dirty]')){
    const target=location.hash;
    // replaceState restores this form without triggering another hashchange.
    // https://developer.mozilla.org/en-US/docs/Web/API/Window/hashchange_event
    history.replaceState(null,'',renderedHash);
    pendingNavigation=target;
    if(el.open)el.close();
    dialog('Hay cambios sin guardar','<p>Puedes seguir editando o descartar los cambios del formulario.</p>',button('Seguir editando','keep-editing')+button('Descartar cambios','discard-edit','primary'));
    return;
  }
  if(el.open)el.close();render();
  const restoring=/^cita\/[^/]+$/.test(from)&&route===state.detailOrigin;
  const opening=/^cita\/[^/]+$/.test(route)&&route!=='cita/nueva';
  window.scrollTo({top:restoring||opening?state.agendaScroll.top:0});
  const scroll=document.querySelector('.week-scroll');
  if(scroll&&(restoring||opening)){scroll.scrollLeft=state.agendaScroll.left;scroll.scrollTop=state.agendaScroll.weekTop;}
});
document.getElementById('appointment-dialog').addEventListener('cancel',event=>{
  event.preventDefault();
  go(state.detailOrigin||state.agendaView);
});
document.getElementById('appointment-dialog').addEventListener('click',event=>{
  const bounds=event.currentTarget.getBoundingClientRect();
  if(event.target===event.currentTarget&&(event.clientX<bounds.left||event.clientX>bounds.right||event.clientY<bounds.top||event.clientY>bounds.bottom))go(state.detailOrigin||state.agendaView);
});
window.addEventListener('beforeunload',event=>{if(document.querySelector('form[data-dirty]')){event.preventDefault();event.returnValue='';}});
document.addEventListener('keydown',event=>{
  const sidebar=document.querySelector('.sidebar.is-open');
  if(sidebar){
    if(event.key==='Escape'){event.preventDefault();toggleNavigation(false);return;}
    if(event.key==='Tab'){
      const controls=Array.from(sidebar.querySelectorAll('a,button')).filter(el=>el.offsetParent!==null);
      const first=controls[0],last=controls[controls.length-1];
      if(event.shiftKey&&document.activeElement===first){event.preventDefault();last.focus();}
      else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first.focus();}
    }
  }
  if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();if(!document.getElementById('dialog').open)searchDialog();}
});
window.addEventListener('resize',()=>{if(window.innerWidth>600&&document.querySelector('.sidebar.is-open'))toggleNavigation(false);});
render();
