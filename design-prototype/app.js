'use strict';

// A separate visual study: all records and outcomes below are fictional.
const TODAY = '2026-10-22';
const icons = {
  calendar: '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M16 3v4M8 3v4M3 11h18"/>',
  people: '<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M22 21v-2a4 4 0 0 0-3-3.87"/><circle cx="9" cy="7" r="4"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>',
  inbox: '<path d="M4 4h16l2 12v4H2v-4L4 4Z"/><path d="M2 16h6l2 3h4l2-3h6"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  message: '<path d="M21 11.5a8.5 8.5 0 0 1-8.5 8.5 9 9 0 0 1-4-.9L3 21l1.9-5.5a9 9 0 0 1-.9-4A8.5 8.5 0 0 1 12.5 3H13a8.5 8.5 0 0 1 8 8v.5Z"/>',
  chart: '<path d="M3 3v18h18M7 16v-3M12 16V8M17 16v-6"/>',
  settings: '<path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M5.6 18.4l2.1-2.1M16.3 7.7l2.1-2.1"/><circle cx="12" cy="12" r="5"/>',
  search: '<circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  left: '<path d="m14 6-6 6 6 6"/>',
  right: '<path d="m10 6 6 6-6 6"/>',
  down: '<path d="m6 9 6 6 6-6"/>',
  more: '<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
  close: '<path d="m6 6 12 12M18 6 6 18"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
  help: '<circle cx="12" cy="12" r="9"/><path d="M9.1 9a3 3 0 0 1 5.8 1c0 2-3 2-3 4M12 17h.01"/>',
  arrow: '<path d="M5 12h14m-5-5 5 5-5 5"/>',
  pin: '<path d="M20 10c0 6-8 11-8 11S4 16 4 10a8 8 0 1 1 16 0Z"/><circle cx="12" cy="10" r="2.5"/>',
  phone: '<path d="m5 3 4 4-2 3a15 15 0 0 0 7 7l3-2 4 4-2 3C9 22 2 15 2 5l3-2Z"/>',
  link: '<path d="M10 13a5 5 0 0 0 7 .2l3-3a5 5 0 0 0-7-7l-2 2M14 11a5 5 0 0 0-7-.2l-3 3a5 5 0 0 0 7 7l2-2"/>',
  device: '<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>',
  shield: '<path d="m12 3 8 4v6c0 5-8 9-8 9s-8-4-8-9V7l8-4Z"/><path d="m8 12 3 3 5-6"/>',
  filter: '<path d="M4 7h16M7 12h10M10 17h4"/>',
  menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
  edit: '<path d="m16 3 5 5-12 12-6 1 1-6L16 3Z M13 6l5 5"/>',
  wifi: '<path d="M2 8a17 17 0 0 1 20 0M5 12a12 12 0 0 1 14 0M8.5 16a6 6 0 0 1 7 0M12 20h.01"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7h.01"/>',
  grid: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>'
};
const icon = name => `<svg class="i" viewBox="0 0 24 24" aria-hidden="true">${icons[name] || icons.calendar}</svg>`;
const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const initials = name => name.trim().split(/\s+/).slice(0,2).map(s=>s[0]).join('');
const avatar = name => `<span class="avatar" aria-hidden="true">${esc(initials(name))}</span>`;
const href = route => `#/${route}`;
const link = (text, route, style='') => `<a class="${style}" href="${href(route)}">${text}</a>`;
const button = (text, action, style='', extra='') => `<button type="button" class="${style}" data-action="${action}" ${extra}>${text}</button>`;
const badge = status => `<span class="badge ${status==='Confirmada'||status==='Entregado'||status==='Atendida'?'success':status==='Solicitud de cambio'?'blue':''}">${esc(status)}</span>`;
const fmt = value => new Intl.DateTimeFormat('es-VE',{day:'numeric',month:'long',timeZone:'UTC'}).format(new Date(value+'T12:00:00Z'));
const fullDate = value => new Intl.DateTimeFormat('es-VE',{weekday:'long',day:'numeric',month:'long',timeZone:'UTC'}).format(new Date(value+'T12:00:00Z'));
const titleCase = text => text.charAt(0).toUpperCase()+text.slice(1);
const patientSeed = [
  {id:'p1',name:'Ana María Rodríguez',phone:'+58 412 000 1001',email:'ana@example.test',contact:'WhatsApp',visits:4},
  {id:'p2',name:'Luis Fernando Pérez',phone:'+58 414 000 1002',email:'luis@example.test',contact:'WhatsApp',visits:2},
  {id:'p3',name:'María Fernanda del Carmen González',phone:'+58 424 000 1003',email:'maria@example.test',contact:'WhatsApp',visits:6},
  {id:'p4',name:'Carlos Méndez',phone:'+58 416 000 1004',email:'carlos@example.test',contact:'WhatsApp',visits:1},
  {id:'p5',name:'Valentina Herrera',phone:'+58 412 000 1005',email:'valentina@example.test',contact:'WhatsApp',visits:3},
  {id:'p6',name:'José Antonio Castillo',phone:'+58 414 000 1006',email:'jose@example.test',contact:'WhatsApp',visits:2},
  {id:'p7',name:'Sofía Rivas',phone:'+58 424 000 1007',email:'sofia@example.test',contact:'WhatsApp',visits:1},
  {id:'p8',name:'Andrés Gutiérrez',phone:'+58 416 000 1008',email:'andres@example.test',contact:'WhatsApp',visits:5}
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
  {id:'a12',patient:'p3',date:'2026-10-19',start:'14:00',end:'14:30',service:'Control de seguimiento',status:'Atendida'}
];
const services = ['Consulta general','Control de seguimiento','Primera consulta'];
const state = {patients:structuredClone(patientSeed),appointments:structuredClone(appointmentSeed),date:TODAY,query:'',filter:'',scenario:'normal',waitlist:[{patient:'p5',service:'Consulta general',preference:'Mañanas · lunes a viernes'},{patient:'p7',service:'Primera consulta',preference:'Tardes · martes o jueves'},{patient:'p8',service:'Control de seguimiento',preference:'Cualquier horario'}],requestResolved:false,hubStatus:'Programada'};
const getPatient = id => state.patients.find(p=>p.id===id)||state.patients[0];
const getAppointment = id => state.appointments.find(a=>a.id===id)||state.appointments[0];
let route = '';
let activeDraft = null;
let pendingNavigation = null;
let toastTimer;

const screens = [
  ['Acceso y preparación',[['acceso','Acceso al escritorio'],['preparacion','Preparación de la consulta']]],
  ['Agenda y citas',[['agenda','Agenda del día'],['agenda/semana','Agenda semanal'],['citas','Lista de citas'],['cita/a1','Detalle de cita'],['cita/nueva','Nueva cita'],['cita/a1/reprogramar','Reprogramar cita']]],
  ['Pacientes',[['pacientes','Directorio de pacientes'],['paciente/p1','Ficha del paciente'],['pacientes/nuevo','Registrar paciente']]],
  ['Seguimiento',[['solicitudes','Solicitudes del paciente'],['solicitud/r1','Revisar una solicitud'],['lista-espera','Lista de espera'],['lista-espera/ofertas','Ofertas de horario'],['comunicaciones','Comunicaciones'],['indicadores','Indicadores']]],
  ['Configuración',[['configuracion/cuenta','Cuenta profesional'],['configuracion/servicios','Servicios'],['configuracion/horarios','Horarios y bloqueos'],['configuracion/comunicaciones','Política de comunicaciones'],['configuracion/equipo','Usuarios y permisos'],['configuracion/dispositivos','Dispositivos']]],
  ['Excepciones y ayuda',[['actividad','Cambios por revisar'],['conflicto/a3','Conflicto de horario'],['diagnostico','Diagnóstico de desarrollo']]],
  ['Patient Hub',[['hub/cita','Cita del paciente'],['hub/confirmada','Confirmación recibida'],['hub/cancelar','Cancelar la cita'],['hub/cancelada','Cita cancelada'],['hub/cambiar','Solicitar cambio'],['hub/solicitud','Solicitud recibida'],['hub/enlace-vencido','Enlace vencido'],['hub/no-disponible','Servicio no disponible']]]
];
window.prototypeScreens = screens.flatMap(([group,items])=>items.map(([path,title])=>({group,path,title})));

function brand(target='agenda'){return `<a href="#/${target}" class="brand" aria-label="Nightingale"><span class="brand-mark" aria-hidden="true">N</span>Nightingale</a>`;}
function prototypeBar(){return `<footer class="prototype-bar"><span>Prototipo de diseño · Datos ficticios</span><div class="tools">${link('Ver todas las pantallas','pantallas')}${button('Probar un escenario','scenarios')}${button('Restablecer','reset')}</div></footer>`;}
function navItem(text,path,ico,active,count=''){return `<a href="${href(path)}" ${active?'aria-current="page"':''}>${icon(ico)}<span>${text}</span>${count?`<span class="count">${count}</span>`:''}</a>`;}
function shell(content,title,section='agenda',crumb=''){
  return `<div class="app-shell"><aside class="sidebar" id="sidebar">${brand()}<div class="practice"><span>Consulta de Elena Arias</span>${icon('down')}</div><nav class="nav" aria-label="Trabajo diario">${navItem('Agenda','agenda','calendar',section==='agenda')}${navItem('Pacientes','pacientes','people',section==='pacientes')}${navItem('Solicitudes','solicitudes','inbox',section==='solicitudes',state.requestResolved?'':'2')}${navItem('Lista de espera','lista-espera','clock',section==='espera')}</nav><div><div class="nav-label">Seguimiento</div><nav class="nav" aria-label="Seguimiento">${navItem('Comunicaciones','comunicaciones','message',section==='comunicaciones')}${navItem('Indicadores','indicadores','chart',section==='indicadores')}</nav></div><div class="sidebar-bottom"><nav class="nav" aria-label="Preferencias">${navItem('Configuración','configuracion/cuenta','settings',section==='configuracion')}</nav><a class="profile" href="#/configuracion/cuenta"><span class="avatar">EA</span><span><strong class="small">Elena Arias</strong><span class="block small subtle">Médica · Cuenta personal</span></span></a></div></aside><div class="workspace"><header class="topbar"><div class="breadcrumb">${button(icon('menu'),'menu','quiet icon mobile-menu','aria-label="Abrir menú" aria-controls="sidebar" aria-expanded="false"')}${icon(section==='pacientes'?'people':section==='solicitudes'?'inbox':section==='configuracion'?'settings':'calendar')}${link(esc(title),section==='configuracion'?'configuracion/cuenta':section==='pacientes'?'pacientes':section==='solicitudes'?'solicitudes':'agenda')}${crumb?`<span class="slash">/</span><span class="muted">${esc(crumb)}</span>`:''}</div><div class="top-actions">${button(`<span>${icon('search')} <span class="search-copy">Buscar en la consulta</span></span><kbd>Ctrl K</kbd>`,'search','search-trigger','aria-label="Buscar paciente o cita"')}${button(icon('help'),'help','quiet icon','aria-label="Ayuda y pantallas"')}</div></header><main id="main">${scenarioNotice()}${content}</main></div></div>${prototypeBar()}`;
}
function head(title,subtitle='',actions=''){return `<div class="page-head"><div><h1>${title}</h1>${subtitle?`<p>${subtitle}</p>`:''}</div>${actions?`<div class="head-actions">${actions}</div>`:''}</div>`;}
function tabs(items,active){return `<nav class="tabs" aria-label="Vistas">${items.map(([name,path])=>`<a href="${href(path)}" ${path===active?'aria-current="page"':''}>${name}</a>`).join('')}</nav>`;}
function notice(title,text,action=''){return `<div class="notice">${icon('info')}<div><strong>${title}</strong>${text?`<p>${text}</p>`:''}</div>${action}</div>`;}
function scenarioNotice(){
  if(state.scenario==='offline')return notice('Sin conexión. Puedes seguir trabajando.','Los cambios se enviarán cuando vuelva la conexión.',link('Ver cambios','actividad','button'));
  if(state.scenario==='conflict')return notice('Una cita necesita tu revisión.','Otro equipo reservó el horario que habías elegido.',link('Revisar','conflicto/a3','button'));
  if(state.scenario==='unavailable')return notice('Algunos cambios todavía no se han enviado.','Tu trabajo se conserva. Volveremos a intentarlo automáticamente.',link('Ver cambios','actividad','button'));
  return '';
}
function empty(title,text,action='',ico='calendar'){return `<div class="empty">${icon(ico)}<h2>${title}</h2><p>${text}</p>${action}</div>`;}
function options(values,selected){return values.map(v=>`<option${v===selected?' selected':''}>${esc(v)}</option>`).join('');}

function appointmentRows(records,compact=false){
  return `<table class="data-table"><thead><tr><th>Horario</th><th>Paciente</th>${compact?'':'<th class="service-col">Servicio</th>'}<th>Estado</th><th class="row-actions"><span class="sr-only">Acciones</span></th></tr></thead><tbody>${records.map(a=>`<tr><td class="time">${esc(a.start)} <span class="subtle">– ${esc(a.end)}</span>${route==='citas'?`<span class="block small subtle">${fmt(a.date)}</span>`:''}</td><td><a class="link-row" href="#/cita/${a.id}">${compact?'':avatar(getPatient(a.patient).name)}<span class="name">${esc(getPatient(a.patient).name)}</span></a></td>${compact?'':`<td class="service-col muted">${esc(a.service)}</td>`}<td>${badge(a.status)}</td><td class="row-actions">${button(icon('more'),`appointment-menu:${a.id}`,'quiet icon',`aria-label="Acciones de la cita de ${esc(getPatient(a.patient).name)}"`)}</td></tr>`).join('')}</tbody></table>`;
}
function agendaToolbar(){return `<div class="toolbar"><div class="toolbar-group"><div class="segmented">${button(icon('left'),'date:-1','','aria-label="Día anterior"')}${button(icon('right'),'date:1','','aria-label="Día siguiente"')}</div>${button('Hoy','today')}<input id="agenda-date" type="date" value="${state.date}" aria-label="Fecha de la agenda"></div><div class="toolbar-group" style="width:auto"><select id="appointment-filter" aria-label="Filtrar por estado"><option value="">Todos los estados</option>${options(['Programada','Confirmada','Atendida','Cancelada'],state.filter)}</select>${button(icon('filter')+' Filtros','filters','quiet')}</div></div>`;}
function availableSlot(date){
  if(date<'2026-10-19'||date>'2026-10-23')return null;
  const dayAppointments=state.appointments.filter(a=>a.date===date&&a.status!=='Cancelada');
  for(const minutes of [540,570,600,630,660,690,840,870,900,930,960,990]){
    const time=m=>String(Math.floor(m/60)).padStart(2,'0')+':'+String(m%60).padStart(2,'0');
    const start=time(minutes),end=time(minutes+30);
    if(!dayAppointments.some(a=>a.start<end&&a.end>start))return {start,end};
  }
  return null;
}
function agenda(detailId=''){
  const records=state.appointments.filter(a=>a.date===state.date&&a.status!=='Cancelada'&&(!state.filter||a.status===state.filter)).sort((a,b)=>a.start.localeCompare(b.start));
  const action=link(icon('plus')+' Nueva cita','cita/nueva','button primary');
  const header=head(state.date===TODAY?'Hoy, jueves 22':titleCase(fullDate(state.date)),'Octubre de 2026 · Hora de Caracas',action);
  const views=tabs([['Día','agenda'],['Semana','agenda/semana'],['Lista de citas','citas']], 'agenda');
  const slot=availableSlot(state.date);
  let table=records.length?appointmentRows(records,!!detailId):empty('No hay citas para mostrar',state.filter?'Prueba con otro estado o elimina los filtros.':state.date===TODAY?'Puedes registrar una cita en un horario disponible.':'La muestra solo incluye datos entre el 19 y el 23 de octubre.',button('Quitar filtros','clear-filter'));
  if(detailId) table=`<div class="split-view"><div class="table-wrap">${table}</div><aside class="detail-panel">${appointmentDetail(getAppointment(detailId))}</aside></div>`;
  else table=`<div class="day-caption"><span>${records.length} citas ${state.filter?'en esta vista':'en el día'}</span><span>Consulta presencial · Consultorio 204</span></div><div class="table-wrap">${table}</div>${slot?`<div class="availability"><span>Horario disponible en el ejemplo</span>${link(icon('plus')+' '+slot.start+' – '+slot.end,'cita/nueva?hora='+slot.start,'button')}</div>`:''}`;
  return shell(header+views+agendaToolbar()+table,'Agenda','agenda',detailId?'Detalle de cita':'');
}
function appointmentDetail(a){
  const p=getPatient(a.patient);
  const done=['Cancelada','Atendida','Ausencia'].includes(a.status);
  return `<div class="detail-top"><span class="small subtle">DETALLE DE CITA</span>${link(icon('close')+'<span class="sr-only">Cerrar detalle</span>','agenda','button')}</div><div class="patient-heading">${avatar(p.name)}<div><h2>${esc(p.name)}</h2><span class="small subtle">${esc(a.service)}</span></div></div>${badge(a.status)}<div class="facts"><div class="fact"><span>Fecha</span><span>${titleCase(fullDate(a.date))}</span></div><div class="fact"><span>Horario</span><span class="mono">${a.start} – ${a.end}</span></div><div class="fact"><span>Lugar</span><span>Consultorio 204<br><span class="small subtle">Centro Médico Los Olivos</span></span></div><div class="fact"><span>Contacto</span><span>${esc(p.phone||'No registrado')}</span></div></div><div class="detail-actions">${done?'':link(icon('calendar')+' Reprogramar',`cita/${a.id}/reprogramar`,'button')}${link('Ver paciente',`paciente/${p.id}`,'button')}</div>${done?'':`<div class="detail-actions" style="margin-top:8px">${a.status==='Programada'?button('Confirmar',`confirm:${a.id}`,'quiet'):''}${button('Registrar atención',`complete:${a.id}`,'quiet')}${button('Registrar ausencia',`absence:${a.id}`,'quiet')}${button('Cancelar cita',`cancel:${a.id}`,'quiet')}</div>`}<div class="section-line"><h3>Comunicaciones</h3>${a.created?'<p class="small subtle">Todavía no hay mensajes para esta cita.</p>':`<div class="link-row small">${icon('message')} Recordatorio por WhatsApp ${badge('Entregado')}</div><p class="small subtle" style="margin-top:7px">21 de octubre · 09:00</p>`}${link('Ver historial de mensajes','comunicaciones','button quiet space-top')}</div><div class="section-line"><h3>Actividad de la cita</h3><ul class="history"><li>Cita registrada por Elena Arias<small>${a.created?'En esta sesión de ejemplo':'19 de octubre · 10:24'}</small></li>${a.status==='Confirmada'?`<li>Asistencia confirmada<small>21 de octubre · 09:18</small></li>`:`<li>Pendiente de confirmación del paciente<small>${a.created?'Todavía no hay comunicaciones para esta cita':'Recordatorio entregado'}</small></li>`}</ul></div>`;
}
function week(){
  let grid='<div></div>';
  ['Lun|19','Mar|20','Mié|21','Jue|22','Vie|23'].forEach((d,i)=>{const [day,num]=d.split('|');grid+=`<div class="day-header ${i===3?'today':''}">${day}<b>${num}</b></div>`;});
  for(let r=0;r<16;r++){
    grid+=`<div class="week-hour" style="grid-row:${r+2};grid-column:1">${r%2===0?`${9+r/2}:00`:''}</div>`;
    for(let c=0;c<5;c++)grid+=`<div class="week-cell" style="grid-row:${r+2};grid-column:${c+2}"></div>`;
  }
  state.appointments.filter(a=>a.status!=='Cancelada').forEach(a=>{const day=Number(a.date.slice(-2))-19;const [h,m]=a.start.split(':').map(Number);const start=(h-9)*2+m/30;const [eh,em]=a.end.split(':').map(Number);const span=Math.max(1,(eh-h)*2+(em-m)/30);if(day>=0&&day<5&&start>=0&&start<16)grid+=`<a class="week-event ${a.status==='Confirmada'?'confirmed':''}" style="grid-column:${day+2};grid-row:${start+2}/span ${span}" href="#/cita/${a.id}" aria-label="${esc(getPatient(a.patient).name)}, ${fmt(a.date)}, ${a.start} a ${a.end}"><b>${esc(getPatient(a.patient).name.split(' ').slice(0,2).join(' '))}</b><small>${a.start} – ${a.end}</small></a>`;});
  return shell(head('La semana','19 – 23 de octubre de 2026 · Hora de Caracas',link(icon('plus')+' Nueva cita','cita/nueva','button primary'))+tabs([['Día','agenda'],['Semana','agenda/semana'],['Lista de citas','citas']],'agenda/semana')+`<div class="toolbar"><span class="small muted">Horario de consulta · 09:00 – 17:00</span>${link('Editar horario','configuracion/horarios','button quiet')}</div><div class="week-scroll"><div class="week">${grid}</div></div>`,'Agenda');
}
function appointmentsList(){
  const records=state.appointments.filter(a=>(!state.filter||a.status===state.filter)&&getPatient(a.patient).name.toLocaleLowerCase('es').includes(state.query.toLocaleLowerCase('es'))).sort((a,b)=>(a.date+a.start).localeCompare(b.date+b.start));
  return shell(head('Todas las citas','Consulta las próximas citas y el historial.',link(icon('plus')+' Nueva cita','cita/nueva','button primary'))+tabs([['Día','agenda'],['Semana','agenda/semana'],['Lista de citas','citas']],'citas')+`<div class="toolbar"><div class="search">${icon('search')}<input id="appointment-search" type="search" placeholder="Buscar por paciente…" value="${esc(state.query)}" aria-label="Buscar citas por paciente"></div><select id="appointment-filter" aria-label="Estado de cita"><option value="">Todos los estados</option>${options(['Programada','Confirmada','Atendida','Cancelada'],state.filter)}</select></div><div class="table-wrap">${records.length?appointmentRows(records):empty('No encontramos citas','Prueba con otro nombre o elimina el filtro.',button('Limpiar búsqueda','clear-search'))}</div><div class="day-caption">${records.length} citas en los datos de ejemplo</div>`,'Agenda');
}

function appointmentEditor(id=''){
  const original=id?getAppointment(id):null;
  const query=new URLSearchParams(location.hash.split('?')[1]||'');
  const start=original?.start||query.get('hora')||availableSlot(state.date)?.start||'10:30';
  const [hours,minutes]=start.split(':').map(Number);
  const endMinutes=hours*60+minutes+30;
  const end=String(Math.floor(endMinutes/60)).padStart(2,'0')+':'+String(endMinutes%60).padStart(2,'0');
  const draft=activeDraft||{patient:original?.patient||query.get('paciente')||'',service:original?.service||services[0],date:original?.date||state.date,start,end:original?.end||end};
  const patientOptions='<option value="">Elige un paciente</option>'+state.patients.map(p=>`<option value="${p.id}" ${draft.patient===p.id?'selected':''}>${esc(p.name)}</option>`).join('');
  const compare=original?notice('Horario actual',`${titleCase(fullDate(original.date))}, ${original.start} – ${original.end}. La nueva fecha necesitará confirmación del paciente.`):'';
  return shell(head(original?'Reprogramar cita':'Nueva cita',original?'Revisa el horario actual y el nuevo antes de guardar.':'Registra una cita en tu agenda.',link('Volver a Agenda','agenda','button quiet'))+`<div class="form-layout"><form id="appointment-form" data-id="${id}">${compare}<fieldset class="form-section"><legend><span class="step-number">1</span> Paciente</legend><label>Nombre del paciente<select name="patient" required>${patientOptions}</select></label>${original?'':button(icon('plus')+' Registrar un paciente nuevo','inline-patient','text space-top')}</fieldset><fieldset class="form-section"><legend><span class="step-number">2</span> Servicio y horario</legend><div class="fields"><label>Servicio<select name="service">${options(services,draft.service)}</select></label><label>Fecha<input type="date" name="date" value="${esc(draft.date)}" required></label><div class="two-fields"><label>Desde<input type="time" name="start" value="${esc(draft.start)}" required></label><label>Hasta<input type="time" name="end" value="${esc(draft.end)}" required></label></div><p class="field-help">Los horarios usan la zona de Caracas.</p></div></fieldset><div id="form-error" class="error" role="alert"></div><div class="form-actions"><button class="primary" type="submit">${original?'Guardar nuevo horario':'Guardar cita'}</button>${link('Descartar','agenda','button')}</div></form><aside class="summary" id="appointment-summary" aria-label="Resumen de la cita">${editorSummary(draft)}</aside></div>`,'Agenda','agenda',original?'Reprogramar':'Nueva cita');
}
function editorSummary(draft){return `<h3>Resumen de la cita</h3><div class="summary-title">${draft.patient?esc(getPatient(draft.patient).name):'Selecciona un paciente'}</div><p>${esc(draft.service)}</p><div class="summary-date"><p>${icon('calendar')} ${titleCase(fullDate(draft.date||TODAY))}</p><p>${icon('clock')} ${esc(draft.start)} – ${esc(draft.end)}</p><p>${icon('pin')} Consultorio 204</p></div><div class="summary-footer">El recordatorio seguirá la política de la consulta. Guardar una cita no equivale a enviar un mensaje.</div>`;}

function patients(){
  const records=state.patients.filter(p=>(p.name+' '+p.phone).toLocaleLowerCase('es').includes(state.query.toLocaleLowerCase('es')));
  return shell(head('Pacientes','La información necesaria para organizar su próxima cita.',link(icon('plus')+' Registrar paciente','pacientes/nuevo','button primary'))+`<div class="toolbar"><div class="search">${icon('search')}<input type="search" id="patient-search" value="${esc(state.query)}" placeholder="Buscar por nombre o teléfono…" aria-label="Buscar pacientes"></div><span class="small subtle">${records.length} pacientes</span></div><div class="table-wrap">${records.length?`<table class="data-table"><thead><tr><th>Nombre</th><th>Teléfono</th><th class="contact-col">Próxima cita</th><th>Estado</th></tr></thead><tbody>${records.map(p=>{const next=state.appointments.find(a=>a.patient===p.id&&a.date>=TODAY&&a.status!=='Cancelada');return `<tr><td><a href="#/paciente/${p.id}" class="link-row">${avatar(p.name)}<span class="name">${esc(p.name)}</span></a></td><td class="muted mono">${esc(p.phone)}</td><td class="contact-col muted">${next?`${fmt(next.date)}, ${next.start}`:'Sin cita'}</td><td>${badge('Activo')}</td></tr>`;}).join('')}</tbody></table>`:empty('No encontramos pacientes','Busca por otro nombre o número de teléfono.',button('Limpiar búsqueda','clear-search'),'people')}</div>`,'Pacientes','pacientes');
}
function patientRecord(id){
  const p=getPatient(id),records=state.appointments.filter(a=>a.patient===p.id);
  return shell(`<div class="record-header">${avatar(p.name)}<div><h1>${esc(p.name)}</h1><p>Paciente desde octubre de 2026</p></div></div><div class="record-grid"><aside class="record-aside"><h3>Información del paciente</h3><div class="facts"><div class="fact"><span>Teléfono</span><span>${esc(p.phone||'No registrado')}</span></div><div class="fact"><span>Correo</span><span>${esc(p.email||'No registrado')}</span></div><div class="fact"><span>Contacto preferido</span><span>${esc(p.contact)}</span></div></div>${button(icon('edit')+' Editar información',`edit-patient:${id}`,'quiet')}</aside><section><div class="page-head"><h2>Citas del paciente</h2>${link(icon('plus')+' Nueva cita',`cita/nueva?paciente=${p.id}`,'button primary')}</div><div class="table-wrap">${appointmentRows(records)}</div><div class="section-line"><h3>Preferencias de comunicación</h3><p class="muted small">WhatsApp para avisos administrativos de sus citas.</p></div></section></div>`,'Pacientes','pacientes',p.name);
}
function patientForm(){return shell(head('Registrar paciente','Solo datos administrativos para gestionar sus citas.',link('Volver a Pacientes','pacientes','button quiet'))+`<form id="patient-form" style="max-width:570px"><div class="fields"><label>Nombre y apellidos<input name="name" autocomplete="off" required placeholder="Nombre del paciente"></label><label>Teléfono de contacto<input name="phone" type="tel" placeholder="+58 …"><small class="muted">Ejemplo visual: los datos mínimos obligatorios aún deben acordarse.</small></label><label>Correo electrónico <span class="small subtle">Opcional</span><input name="email" type="email" placeholder="nombre@example.test"></label><label class="check-line"><input name="contact" type="checkbox" checked> Contacto por WhatsApp para avisos de citas</label></div><div id="form-error" class="error" role="alert"></div><div class="form-actions"><button type="submit" class="primary">Registrar paciente</button>${link('Descartar','pacientes','button')}</div></form>`,'Pacientes','pacientes','Registrar');}

function requests(){return shell(head('Solicitudes','Cambios que los pacientes han pedido desde su enlace de cita.')+tabs([['Por revisar','solicitudes']],'solicitudes')+(state.requestResolved?empty('Todo revisado','No hay más solicitudes pendientes en este ejemplo.',link('Volver a Agenda','agenda','button'),'inbox'):`<div class="list-item"><div><a href="#/solicitud/r1" class="link-row">${avatar(getPatient('p2').name)}<strong>${getPatient('p2').name}</strong>${badge('Solicitud de cambio')}</a><p>Solicita cambiar su cita del 22 de octubre, 09:30.</p><small>Recibida ayer, 16:20 · Cita sin modificar</small></div>${link('Revisar solicitud','solicitud/r1','button')}</div><div class="list-item"><div><a href="#/solicitud/r2" class="link-row">${avatar(getPatient('p5').name)}<strong>${getPatient('p5').name}</strong>${badge('Solicitud de cambio')}</a><p>Solicita cambiar su cita del 22 de octubre, 11:30.</p><small>Recibida hoy, 08:10 · Cita sin modificar</small></div>${link('Revisar solicitud','solicitud/r2','button')}</div>`),'Solicitudes','solicitudes');}
function requestDetail(id){const a=getAppointment(id==='r2'?'a5':'a2'),p=getPatient(a.patient);return shell(head('Solicitud de cambio','El paciente pide otra fecha. Su cita mantiene el horario actual.',link('Volver a Solicitudes','solicitudes','button quiet'))+`<div class="form-layout"><section><div class="patient-heading">${avatar(p.name)}<div><h2>${esc(p.name)}</h2><p class="small subtle">${esc(p.phone)}</p></div></div><div class="section-line"><h3>Lo que pide el paciente</h3><p class="muted" style="margin-top:12px">«No podré asistir ese día. ¿Podrían ofrecerme otra fecha?»</p><p class="small subtle space-top">Solicitud de ejemplo · 21 de octubre, 16:20</p></div><div class="section-line"><h3>Próximo paso</h3><p class="muted small">Busca otro horario y revisa el cambio antes de registrarlo. La solicitud por sí sola no reserva una hora.</p><div class="detail-actions space-top">${link(icon('calendar')+' Elegir nuevo horario',`cita/${a.id}/reprogramar`,'button primary')}</div></div></section><aside class="summary"><h3>Cita actual</h3><div class="summary-title">${fmt(a.date)}</div><p>${a.start} – ${a.end}</p><p>${a.service}</p><p>Consultorio 204</p><div class="summary-footer">Las reglas para rechazar o retirar solicitudes siguen pendientes de definición; esta muestra no las fija.</div></aside></div>`,'Solicitudes','solicitudes','Revisar');}

function waitlist(offers=false){return shell(head('Lista de espera','Encuentra pacientes compatibles cuando quede un horario libre.',button(icon('plus')+' Añadir paciente','add-waitlist','primary'))+tabs([['En espera','lista-espera'],['Ofertas','lista-espera/ofertas']],offers?'lista-espera/ofertas':'lista-espera')+(offers?`<div class="list-item"><div><strong>${getPatient('p5').name}</strong><p>Consulta general · 23 de octubre, 10:30 – 11:00</p><small>Oferta de ejemplo. Vence el 22 de octubre a las 18:00.</small></div>${badge('Esperando respuesta')}</div>`:`<div class="table-wrap"><table class="data-table"><thead><tr><th>Paciente</th><th>Servicio</th><th>Preferencia</th><th></th></tr></thead><tbody>${state.waitlist.map((w,i)=>`<tr><td><a class="link-row" href="#/paciente/${w.patient}">${avatar(getPatient(w.patient).name)}<span class="name">${esc(getPatient(w.patient).name)}</span></a></td><td class="muted">${w.service}</td><td class="muted">${w.preference}</td><td>${button('Revisar horarios',`waitlist-slot:${i}`,'quiet')}</td></tr>`).join('')}</tbody></table></div><p class="small subtle space-top">El orden de esta lista no fija una prioridad de asignación.</p>`),'Lista de espera','espera');}

function communications(){const data=[['p1','Recordatorio de cita','21 oct · 09:00','Entregado'],['p2','Recordatorio de cita','21 oct · 09:30','Entregado'],['p4','Confirmación de asistencia','21 oct · 17:10','Respondido'],['p5','Recordatorio de cita','22 oct · 08:00','Programado'],['p6','Recordatorio de cita','22 oct · 08:30','Revisar teléfono']];return shell(head('Comunicaciones','Recordatorios y respuestas relacionados con las citas.',link(icon('settings')+' Política de avisos','configuracion/comunicaciones','button'))+`<div class="toolbar"><span class="small muted">Mensajes recientes · WhatsApp</span><span class="small subtle">Datos de ejemplo</span></div><div class="table-wrap"><table class="data-table"><thead><tr><th>Paciente</th><th>Comunicación</th><th>Fecha</th><th>Estado</th><th></th></tr></thead><tbody>${data.map(([p,type,date,status])=>`<tr><td><a href="#/paciente/${p}" class="link-row">${avatar(getPatient(p).name)}<span class="name">${esc(getPatient(p).name)}</span></a></td><td class="muted">${type}</td><td class="muted mono">${date}</td><td>${badge(status)}</td><td>${button('Ver detalle',`message:${p}`,'quiet')}</td></tr>`).join('')}</tbody></table></div>`,'Comunicaciones','comunicaciones');}
function metrics(){return shell(head('Indicadores','Citas y resultados administrativos de la consulta.')+`<div class="toolbar"><span class="button">1 – 21 de octubre de 2026 ${icon('down')}</span><span class="small subtle">Informe ficticio · Hora de Caracas</span></div><div class="metric-row"><div class="metric"><p>Citas registradas</p><div class="value mono">64</div><small>Durante el periodo</small></div><div class="metric"><p>Asistencia</p><div class="value mono">90 %</div><small>45 atendidas de 50 finalizadas</small></div><div class="metric"><p>Ausencias sin aviso</p><div class="value mono">5</div><small>10 % de las 50 finalizadas</small></div><div class="metric"><p>Horarios recuperados</p><div class="value mono">4</div><small>Desde la lista de espera</small></div></div><div class="section-title"><h2>Citas registradas por semana</h2><span class="small subtle">64 citas en total</span></div><div class="chart" role="img" aria-label="Citas por semana: 18 del 1 al 7, 22 del 8 al 14 y 24 del 15 al 21 de octubre"><div class="bar-group"><span>18</span><div class="bar" style="height:60%"></div><span>1 – 7 oct</span></div><div class="bar-group"><span>22</span><div class="bar" style="height:73.33%"></div><span>8 – 14 oct</span></div><div class="bar-group"><span>24</span><div class="bar" style="height:80%"></div><span>15 – 21 oct</span></div></div><div class="table-wrap space-bottom"><table class="data-table"><thead><tr><th>Seguimiento del periodo</th><th>Total</th><th>Sobre las citas registradas</th></tr></thead><tbody><tr><td>Confirmaciones recibidas</td><td>48</td><td>48 de 64 · 75 %</td></tr><tr><td>Cancelaciones</td><td>8</td><td>8 de 64 · 12,5 %</td></tr></tbody></table></div><details><summary class="small muted">Cómo se calculan estos indicadores</summary><div class="space-top"><table class="data-table"><thead><tr><th>Indicador</th><th>Cálculo del ejemplo</th></tr></thead><tbody><tr><td>Asistencia</td><td>45 atendidas / (45 atendidas + 5 ausencias)</td></tr><tr><td>Ausencias</td><td>5 ausencias / 50 citas finalizadas</td></tr><tr><td>Citas restantes</td><td>8 canceladas y 6 aún programadas, de las 64 registradas</td></tr></tbody></table></div></details>`,'Indicadores','indicadores');}

function activity(){return shell(head('Cambios por revisar','Tu trabajo permanece guardado en este equipo.',link('Volver a Agenda','agenda','button quiet'))+(state.scenario==='normal'?empty('No hay cambios por revisar','Los cambios del ejemplo se han guardado. Solo verás avisos aquí si algo necesita atención.',link('Ver Agenda','agenda','button'),'check'):`<div class="list-item"><div><strong>${getPatient('p3').name}</strong><p>Cita del 22 de octubre, 10:00 – 10:30</p><small>${state.scenario==='conflict'?'El horario cambió en otro equipo.':'El envío se reanudará automáticamente al recuperar el servicio.'}</small></div>${state.scenario==='conflict'?link('Revisar horario','conflicto/a3','button'):badge('Guardada en este equipo')}</div>`),'Ayuda');}
function conflict(){return shell(head('Revisar un horario','Otro equipo reservó el intervalo que elegiste.',link('Volver a Agenda','agenda','button quiet'))+`<div class="patient-heading">${avatar(getPatient('p3').name)}<div><h2>${getPatient('p3').name}</h2><p class="small subtle">Consulta general</p></div></div><div class="compare"><div><h3>Tu propuesta</h3><p>22 de octubre · 10:00 – 10:30</p><p class="small muted space-top">La conservamos para que puedas elegir otra hora.</p></div><div><h3>Horario compartido</h3><p>Ese intervalo ya está ocupado.</p><p class="small muted space-top">No se ha confirmado la reserva de este paciente en ese horario.</p></div></div><div class="detail-actions">${link('Elegir otro horario','cita/a3/reprogramar','button primary')}${button('Descartar mi propuesta','discard-conflict')}</div>`,'Agenda','agenda','Revisar horario');}

const settingsItems=[['Cuenta','cuenta'],['Servicios','servicios'],['Horarios y bloqueos','horarios'],['Comunicaciones','comunicaciones'],['Usuarios y permisos','equipo'],['Dispositivos','dispositivos']];
function settings(section){
  let content='';
  if(section==='cuenta')content=`<form id="settings-form"><section class="settings-section"><h2>Datos de la consulta</h2><p>Información administrativa que aparece en las citas.</p><div class="fields"><label>Nombre de la consulta<input name="practice" value="Consulta de Elena Arias" required></label><div class="two-fields"><label>Profesional<input name="professional" value="Dra. Elena Arias" required></label><label>Especialidad<input name="specialty" value="Medicina general" required></label></div><label>Lugar de atención<input name="location" value="Centro Médico Los Olivos · Consultorio 204" required></label></div></section><section class="settings-section"><h2>Hora y formato</h2><p>Los horarios se presentan según esta zona.</p><div class="two-fields"><label>Zona horaria<input value="América / Caracas (UTC −04:00)" readonly></label><label>Formato de hora<select><option>24 horas</option><option>12 horas</option></select></label></div></section><button type="submit" class="primary">Guardar cambios</button></form>`;
  if(section==='servicios')content=`<div class="page-head"><div><h2>Servicios de la consulta</h2><p>La duración del ejemplo se puede revisar antes de guardar una cita.</p></div>${button(icon('plus')+' Añadir','add-service')}</div>${services.map((s,i)=>`<div class="list-item"><div><strong>${s}</strong><p>${i===2?'30':'30'} minutos · Presencial</p></div><div class="item-actions">${badge('Activo')}${button(icon('edit'),`edit-service:${i}`,'quiet icon',`aria-label="Editar ${s}"`)}</div></div>`).join('')}<p class="small subtle space-top">Cambiar un servicio no modifica citas ya registradas.</p>`;
  if(section==='horarios')content=`<section class="settings-section"><h2>Horario habitual</h2><p>Disponibilidad de ejemplo para la consulta presencial.</p><table class="data-table"><thead><tr><th>Día</th><th>Mañana</th><th>Tarde</th></tr></thead><tbody>${['Lunes','Martes','Miércoles','Jueves','Viernes'].map(d=>`<tr><td>${d}</td><td>09:00 – 12:00</td><td>14:00 – 17:00</td></tr>`).join('')}</tbody></table>${button('Editar horario','edit-hours','space-top')}</section><div class="section-title"><h2>Bloqueos y excepciones</h2>${button(icon('plus')+' Añadir bloqueo','add-block')}</div><div class="list-item"><div><strong>Reunión administrativa</strong><p>Viernes, 23 de octubre · 12:00 – 14:00</p></div>${badge('Bloqueado')}</div>`;
  if(section==='comunicaciones')content=`<form id="settings-form"><section class="settings-section"><h2>Avisos por WhatsApp</h2><p>Configuración de muestra. Los plazos definitivos deben acordarse con la consulta.</p><div class="fields"><label class="check-line"><input type="checkbox" checked> Recordatorio previo a la cita</label><label>Anticipación de ejemplo<select><option>24 horas antes</option><option>48 horas antes</option></select></label><label class="check-line"><input type="checkbox" checked> Incluir acceso a la cita del paciente</label></div></section><section class="settings-section"><h2>Cambios y cancelaciones</h2><p>El paciente verá las acciones que permita la política vigente.</p><p class="small muted">El diseño no fija un plazo de cancelación ni una verificación adicional del enlace.</p></section><button type="submit" class="primary">Guardar configuración de ejemplo</button></form>`;
  if(section==='equipo')content=`<div class="page-head"><div><h2>Personas de la consulta</h2><p>Cada persona usa su propia cuenta.</p></div>${button(icon('plus')+' Invitar asistente','invite')}</div><div class="list-item"><div class="link-row">${avatar('Elena Arias')}<div><strong>Elena Arias</strong><p>elena@example.test</p></div></div>${badge('Médica')}</div><div class="list-item"><div class="link-row">${avatar('Lucía Torres')}<div><strong>Lucía Torres</strong><p>lucia@example.test</p></div></div>${badge('Asistente')}</div><p class="small subtle space-top">Cambiar permisos requiere conexión y autorización.</p>`;
  if(section==='dispositivos')content=`<h2>Equipos autorizados</h2><p class="muted small" style="margin-top:8px">Revisa desde qué equipos se accede a la consulta.</p><div class="list-item"><div class="link-row">${icon('device')}<div><strong>Equipo de Elena</strong><p>Windows · Este equipo</p></div></div>${badge('Actual')}</div><div class="list-item"><div class="link-row">${icon('device')}<div><strong>Recepción</strong><p>Windows · Último acceso: hoy, 08:35</p></div></div>${button('Revisar acceso','review-device','quiet')}</div>`;
  return shell(head('Configuración','Preferencias y datos de tu consulta.')+`<div class="settings-layout"><nav class="settings-nav" aria-label="Secciones de configuración">${settingsItems.map(([name,id])=>`<a href="#/configuracion/${id}" ${section===id?'aria-current="page"':''}>${name}</a>`).join('')}</nav><div>${content}</div></div>`,'Configuración','configuracion');
}
function diagnostics(){return shell(head('Diagnóstico','Información de ejemplo para revisar la propuesta visual.')+notice('Esta pantalla no consulta el escritorio real.','Las versiones y el estado de servicios solo se verificarán en la aplicación Tauri.')+`<table class="data-table"><tbody><tr><td>Vista</td><td>Prototipo de diseño</td></tr><tr><td>Datos</td><td>Ficticios, en memoria</td></tr><tr><td>Zona horaria del ejemplo</td><td>América / Caracas</td></tr><tr><td>Conexiones a servicios</td><td>Ninguna</td></tr></tbody></table>`,'Ayuda');}

function auth(setup=false){return `<main id="main" class="auth-page"><section class="auth-card">${brand()}${setup?`<h1>Tu consulta, lista para empezar</h1><p>Revisa estos datos antes de registrar la primera cita.</p><ul class="checklist" style="padding:0;list-style:none"><li><span class="check-icon">${icon('check')}</span><span>Consulta de Elena Arias<br><span class="small subtle">Medicina general · Cuenta profesional</span></span></li><li><span class="check-icon">${icon('check')}</span><span>Hora de Caracas<br><span class="small subtle">UTC −04:00</span></span></li><li><span class="check-icon">${icon('check')}</span><span>Servicios y horario de ejemplo<br><span class="small subtle">Podrás revisarlos en Configuración.</span></span></li></ul>${link('Abrir mi agenda '+icon('arrow'),'agenda','button primary full space-top')}`:`<h1>Entra a tu consulta</h1><p>Gestiona tus citas y mantén cerca lo que importa hoy.</p><label>Cuenta profesional<input value="elena@example.test" readonly></label>${link('Continuar con mi cuenta '+icon('arrow'),'preparacion','button primary full space-top')}<p class="auth-foot">Recorrido visual de ejemplo. El método de acceso definitivo aún no está elegido.</p>`}</section></main>${prototypeBar()}`;}

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
  return `<main id="main" class="hub-page"><div class="hub-brand">${brand('hub/cita')}</div><section class="hub-card">${content}</section><p class="hub-foot">Solo la información administrativa de tu cita.<br>Prototipo: las acciones y respuestas son simuladas.</p></main>${prototypeBar()}`;
}
function catalog(){return `<main id="main" class="catalog">${brand()}<h1>Pantallas de Nightingale</h1><p>Un recorrido por la propuesta de escritorio y el acceso del paciente. Todas las pantallas comparten el mismo sistema visual y datos de ejemplo.</p><div class="head-actions space-bottom">${link('Explorar Agenda '+icon('arrow'),'agenda','button primary')}${link('Abrir vista del paciente','hub/cita','button')}</div><div class="catalog-grid">${screens.map(([group,items])=>`<section class="catalog-group"><h2>${group}</h2>${items.map(([path,title])=>link(esc(title)+icon('right'),path)).join('')}</section>`).join('')}</div></main>${prototypeBar()}`;}

function render(){
  route=(location.hash.replace(/^#\/?/,'')||'agenda').split('?')[0];
  let html='';
  if(route==='pantallas')html=catalog();
  else if(route==='acceso'||route==='preparacion')html=auth(route==='preparacion');
  else if(route.startsWith('hub/'))html=hub(route.split('/')[1]);
  else if(route==='agenda/semana')html=week();
  else if(route==='citas')html=appointmentsList();
  else if(route==='cita/nueva')html=appointmentEditor();
  else if(/^cita\/[^/]+\/reprogramar$/.test(route))html=appointmentEditor(route.split('/')[1]);
  else if(route.startsWith('cita/'))html=agenda(route.split('/')[1]);
  else if(route==='pacientes/nuevo')html=patientForm();
  else if(route==='pacientes')html=patients();
  else if(route.startsWith('paciente/'))html=patientRecord(route.split('/')[1]);
  else if(route==='solicitudes')html=requests();
  else if(route.startsWith('solicitud/'))html=requestDetail(route.split('/')[1]);
  else if(route.startsWith('lista-espera'))html=waitlist(route.endsWith('/ofertas'));
  else if(route==='comunicaciones')html=communications();
  else if(route==='indicadores')html=metrics();
  else if(route==='actividad')html=activity();
  else if(route.startsWith('conflicto/'))html=conflict();
  else if(route.startsWith('configuracion/'))html=settings(route.split('/')[1]);
  else if(route==='diagnostico')html=diagnostics();
  else html=agenda();
  document.getElementById('app').innerHTML=html;
  const title=document.querySelector('h1')?.textContent||'Agenda';
  document.title=`Nightingale · ${title}`;
}
function go(path){location.hash=href(path);}
function toast(text){const el=document.getElementById('toast');el.textContent=text;el.classList.add('visible');clearTimeout(toastTimer);toastTimer=setTimeout(()=>el.classList.remove('visible'),4500);}
function dialog(title,body,actions=''){const el=document.getElementById('dialog');el.innerHTML=`<h2 id="dialog-title">${title}</h2>${body}<div class="dialog-actions">${button('Cerrar','close-dialog')}${actions}</div>`;el.showModal();}
function closeDialog(){document.getElementById('dialog').close();}
function errorMessage(text){const el=document.getElementById('form-error');el.textContent=text;el.scrollIntoView({block:'nearest'});}
function formValues(form){return Object.fromEntries(new FormData(form));}
function editPatient(id){const p=getPatient(id);dialog('Editar información',`<form id="edit-patient-form" data-id="${id}"><div class="fields"><label>Nombre y apellidos<input name="name" value="${esc(p.name)}" required></label><label>Teléfono<input name="phone" value="${esc(p.phone)}"></label><label>Correo electrónico<input type="email" name="email" value="${esc(p.email||'')}"></label></div><div class="form-actions"><button type="submit" class="primary">Guardar cambios</button></div></form>`);}
function searchDialog(){dialog('Buscar en la consulta',`<input id="global-search" type="search" aria-label="Nombre de paciente" placeholder="Buscar paciente o cita…" autofocus><div class="search-results" id="global-results">${searchResults('')}</div>`);}
function searchResults(query){return state.patients.filter(p=>p.name.toLocaleLowerCase('es').includes(query.toLocaleLowerCase('es'))).map(p=>`<a href="#/paciente/${p.id}" data-close="true"><span class="link-row">${avatar(p.name)}${esc(p.name)}</span><span class="small subtle">Paciente · ${esc(p.phone||'No registrado')}</span></a>`).join('')||'<p class="small muted">No hay coincidencias en los datos de ejemplo.</p>';}

document.addEventListener('click',event=>{
  const el=event.target.closest('[data-action]');
  if(event.target.closest('[data-close]'))closeDialog();
  if(!el)return;
  const [action,id]=el.dataset.action.split(':');
  if(action==='close-dialog')closeDialog();
  else if(action==='menu'){const open=document.getElementById('sidebar').classList.toggle('is-open');el.setAttribute('aria-expanded',String(open));}
  else if(action==='today'){state.date=TODAY;state.filter='';render();}
  else if(action==='date'){const date=new Date(state.date+'T12:00:00Z');date.setUTCDate(date.getUTCDate()+Number(id));state.date=date.toISOString().slice(0,10);render();}
  else if(action==='clear-filter'){state.filter='';render();}
  else if(action==='clear-search'){state.query='';state.filter='';render();}
  else if(action==='search')searchDialog();
  else if(action==='help')dialog('Ayuda del prototipo',`<p>Explora la agenda, abre una cita o revisa la ficha de un paciente. Los formularios usan datos ficticios y no contactan con ningún servicio.</p>${link('Ver todas las pantallas','pantallas','button')} ${link('Diagnóstico de desarrollo','diagnostico','button')}`);
  else if(action==='filters')dialog('Filtrar la agenda',`<p>Filtra las citas por su estado. No se ocultan otras citas para calcular disponibilidad.</p><label>Estado<select id="dialog-filter"><option value="">Todos los estados</option>${options(['Programada','Confirmada','Atendida','Cancelada'],state.filter)}</select></label>`,button('Aplicar filtro','apply-filter','primary'));
  else if(action==='apply-filter'){state.filter=document.getElementById('dialog-filter').value;closeDialog();render();}
  else if(action==='appointment-menu'){const a=getAppointment(id);dialog(getPatient(a.patient).name,`<p>${fmt(a.date)} · ${a.start} – ${a.end}</p><div class="detail-actions">${link('Ver cita',`cita/${id}`,'button')}${link('Reprogramar',`cita/${id}/reprogramar`,'button')}</div>`,button('Cancelar cita',`cancel:${id}`));}
  else if(action==='cancel'){if(document.getElementById('dialog').open)closeDialog();const a=getAppointment(id);dialog('¿Cancelar esta cita?',`<p><strong>${esc(getPatient(a.patient).name)}</strong><br>${fmt(a.date)} · ${a.start} – ${a.end}</p><p>Se conservará el historial. El horario podrá ofrecerse a otra persona cuando se acepte la cancelación.</p>`,button('Cancelar cita',`do-cancel:${id}`,'primary'));}
  else if(action==='do-cancel'){getAppointment(id).status='Cancelada';closeDialog();render();toast('Cita cancelada en el ejemplo.');}
  else if(action==='absence')dialog('Registrar ausencia','<p>En este ejemplo, el personal ya comprobó que pasó la hora de la cita y el paciente no asistió. El margen de espera definitivo sigue pendiente.</p>',button('Registrar ausencia',`do-absence:${id}`,'primary'));
  else if(action==='do-absence'){getAppointment(id).status='Ausencia';closeDialog();render();toast('Ausencia registrada en el ejemplo.');}
  else if(action==='confirm'||action==='complete'){getAppointment(id).status=action==='confirm'?'Confirmada':'Atendida';render();toast(action==='confirm'?'Asistencia confirmada en el ejemplo.':'Atención registrada en el ejemplo.');}
  else if(action==='edit-patient')editPatient(id);
  else if(action==='inline-patient'){activeDraft=formValues(document.getElementById('appointment-form'));dialog('Registrar paciente',`<form id="inline-patient-form"><div class="fields"><label>Nombre y apellidos<input name="name" required autofocus></label><label>Teléfono<input name="phone" type="tel"></label></div><div class="form-actions"><button type="submit" class="primary">Registrar y volver a la cita</button></div></form>`);}
  else if(action==='add-waitlist')dialog('Añadir a la lista de espera',`<form id="waitlist-form"><div class="fields"><label>Paciente<select name="patient">${state.patients.map(p=>`<option value="${p.id}">${esc(p.name)}</option>`).join('')}</select></label><label>Servicio<select name="service">${options(services)}</select></label><label>Preferencia de horario<input name="preference" placeholder="Por ejemplo, por las mañanas" required></label></div><div class="form-actions"><button type="submit" class="primary">Añadir paciente</button></div></form>`);
  else if(action==='waitlist-slot'){const w=state.waitlist[Number(id)];dialog('Revisar un horario',`<p><strong>${esc(getPatient(w.patient).name)}</strong><br>${w.service}<br>${w.preference}</p><p>La oferta y su respuesta no confirman una reserva. La política de prioridad sigue pendiente.</p>`,link('Revisar nueva cita',`cita/nueva?paciente=${w.patient}`,'button primary'));}
  else if(action==='message')dialog('Detalle de comunicación',`<p><strong>${esc(getPatient(id).name)}</strong><br>WhatsApp · Recordatorio administrativo</p><p>${id==='p6'?'El número no pudo validarse en este ejemplo. Revisa el contacto antes de solicitar otro intento.':'Resultado recibido: entregado. La entrega no confirma asistencia.'}</p>`,link('Ver paciente',`paciente/${id}`,'button'));
  else if(action==='add-service'||action==='edit-service')dialog(action==='add-service'?'Añadir servicio':'Editar servicio',`<form id="service-form" data-index="${id??''}"><div class="fields"><label>Nombre<input name="name" value="${id!==undefined?esc(services[Number(id)]):''}" required></label><label>Duración de ejemplo (minutos)<input name="duration" type="number" min="5" value="30" required></label></div><p>Esta configuración no cambia las citas existentes.</p><div class="form-actions"><button class="primary" type="submit">Guardar servicio</button></div></form>`);
  else if(action==='edit-hours'||action==='add-block')dialog(action==='add-block'?'Añadir un bloqueo':'Editar horario',`<form id="hours-form"><div class="fields"><label>${action==='add-block'?'Motivo':'Día'}<input name="label" value="${action==='add-block'?'':'Lunes a viernes'}" required></label><div class="two-fields"><label>Desde<input type="time" value="09:00" required></label><label>Hasta<input type="time" value="12:00" required></label></div></div><p>Un cambio de horario no debe cancelar citas existentes sin una decisión explícita.</p><div class="form-actions"><button type="submit" class="primary">Guardar ejemplo</button></div></form>`);
  else if(action==='invite')dialog('Invitar a un asistente',`<form id="invite-form"><label>Correo de la persona<input type="email" required placeholder="asistente@example.test"></label><p>El envío no se ejecuta. Los permisos concretos deben definirse antes de implementar el acceso.</p><div class="form-actions"><button type="submit" class="primary">Ver invitación de ejemplo</button></div></form>`);
  else if(action==='review-device')dialog('Acceso del equipo Recepción','<p>La revocación real requiere conexión y autorización. Un equipo desconectado no recibe el cambio de inmediato.</p><p>Esta muestra permite revisar la información del equipo; no modifica permisos.</p>');
  else if(action==='discard-conflict')dialog('¿Descartar tu propuesta?','<p>El horario aceptado en la consulta se mantiene. Solo se retira tu propuesta pendiente en este ejemplo.</p>',button('Descartar propuesta','do-discard-conflict','primary'));
  else if(action==='do-discard-conflict'){state.scenario='normal';closeDialog();go('agenda');toast('Propuesta descartada en el ejemplo.');}
  else if(action==='hub-confirm'){state.hubStatus='Confirmada';go('hub/confirmada');}
  else if(action==='hub-cancel'){state.hubStatus='Cancelada';go('hub/cancelada');}
  else if(action==='hub-retry')toast('Ejemplo: el resultado sigue pendiente de comprobar.');
  else if(action==='scenarios')dialog('Probar un escenario',`<p>Estos controles pertenecen al prototipo, no a la aplicación.</p><div class="fields">${[['normal','Funcionamiento normal'],['offline','Sin conexión'],['unavailable','Servicio temporalmente no disponible'],['conflict','Conflicto de horario']].map(([value,label])=>`<label class="radio-line"><input type="radio" name="scenario" value="${value}" ${state.scenario===value?'checked':''}>${label}</label>`).join('')}</div>`,button('Mostrar escenario','apply-scenario','primary'));
  else if(action==='apply-scenario'){state.scenario=document.querySelector('input[name=scenario]:checked').value;closeDialog();render();}
  else if(action==='reset')dialog('Restablecer los datos de ejemplo','<p>Se retirarán las ediciones hechas durante esta visita al prototipo.</p>',button('Restablecer','do-reset','primary'));
  else if(action==='do-reset'){location.reload();}
  else if(action==='keep-editing'){pendingNavigation=null;closeDialog();}
  else if(action==='discard-edit'){const target=pendingNavigation;pendingNavigation=null;activeDraft=null;document.querySelector('form[data-dirty]')?.removeAttribute('data-dirty');closeDialog();location.hash=target;}
});

document.addEventListener('input',event=>{
  const el=event.target;
  if(el.closest('#appointment-form')){el.closest('form').dataset.dirty='true';const values=formValues(el.closest('form'));document.getElementById('appointment-summary').innerHTML=editorSummary(values);}
  if(el.closest('#patient-form'))el.closest('form').dataset.dirty='true';
  if(el.id==='global-search')document.getElementById('global-results').innerHTML=searchResults(el.value);
  if(['patient-search','appointment-search'].includes(el.id)){const selection=el.selectionStart;state.query=el.value;render();const replacement=document.getElementById(el.id);replacement.focus();replacement.setSelectionRange(selection,selection);}
});
document.addEventListener('change',event=>{
  if(event.target.id==='agenda-date'){state.date=event.target.value||TODAY;render();}
  if(event.target.id==='appointment-filter'){state.filter=event.target.value;render();}
});
document.addEventListener('submit',event=>{
  const form=event.target;event.preventDefault();
  const values=formValues(form);
  if(form.id==='appointment-form'){
    if(!values.patient)return errorMessage('Elige un paciente para la cita.');
    if(values.end<=values.start)return errorMessage('La hora de fin debe ser posterior a la de inicio.');
    const conflict=state.appointments.some(a=>a.id!==form.dataset.id&&a.date===values.date&&a.status!=='Cancelada'&&a.start<values.end&&a.end>values.start);
    if(conflict)return errorMessage('Ese intervalo coincide con otra cita. Revisa el horario; tus datos se conservan.');
    if(values.start<'09:00'||values.end>'17:00'||(values.start<'14:00'&&values.end>'12:00'))return errorMessage('Elige un intervalo dentro del horario de consulta: 09:00–12:00 o 14:00–17:00.');
    const original=form.dataset.id?getAppointment(form.dataset.id):null;
    if(original){original.previous={date:original.date,start:original.start,end:original.end};Object.assign(original,values,{status:'Programada'});}
    else state.appointments.push({...values,id:'a'+(state.appointments.length+1),status:'Programada',created:true});
    state.date=values.date;form.removeAttribute('data-dirty');activeDraft=null;go('agenda');toast(state.scenario==='normal'?'Cita guardada en el ejemplo.':'Cita guardada en este equipo. El envío queda pendiente.');
  }else if(form.id==='patient-form'||form.id==='inline-patient-form'){
    const p={...values,id:'p'+(state.patients.length+1),contact:'WhatsApp',visits:0};state.patients.push(p);
    if(form.id==='inline-patient-form'){activeDraft.patient=p.id;closeDialog();render();document.getElementById('appointment-form').dataset.dirty='true';}
    else{form.removeAttribute('data-dirty');go('paciente/'+p.id);}
    toast('Paciente registrado en el ejemplo.');
  }else if(form.id==='edit-patient-form'){Object.assign(getPatient(form.dataset.id),values);closeDialog();render();toast('Información actualizada en el ejemplo.');}
  else if(form.id==='waitlist-form'){state.waitlist.push(values);closeDialog();render();toast('Paciente añadido a la lista de espera.');}
  else if(form.id==='hub-request-form')go('hub/solicitud');
  else if(form.id==='settings-form')toast('Preferencias revisadas. Esta muestra no cambia la configuración real.');
  else if(form.id==='service-form'){if(form.dataset.index!=='')services[Number(form.dataset.index)]=values.name;else services.push(values.name);closeDialog();render();toast('Servicio guardado en el ejemplo.');}
  else if(form.id==='hours-form'||form.id==='invite-form'){closeDialog();toast('Ejemplo revisado. No se envían invitaciones ni se cambia la agenda real.');}
});
document.addEventListener('click',event=>{
  const anchor=event.target.closest('a[href^="#/"]');
  if(!anchor)return;
  if(document.getElementById('dialog').open){if(document.getElementById('dialog').querySelector('form[data-dirty]'))return;closeDialog();}
  const dirty=document.querySelector('form[data-dirty]');
  if(dirty&&anchor.hash!==location.hash){event.preventDefault();pendingNavigation=anchor.hash;dialog('Hay cambios sin guardar','<p>Puedes seguir editando o descartar los cambios del formulario.</p>',button('Seguir editando','keep-editing')+button('Descartar cambios','discard-edit','primary'));return;}
  if(!anchor.hash.includes('cita/nueva'))activeDraft=null;
});
window.addEventListener('hashchange',()=>{const el=document.getElementById('dialog');if(el.open)el.close();render();window.scrollTo({top:0});});
window.addEventListener('beforeunload',event=>{if(document.querySelector('form[data-dirty]')){event.preventDefault();event.returnValue='';}});
document.addEventListener('keydown',event=>{if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();if(!document.getElementById('dialog').open)searchDialog();}});
render();
