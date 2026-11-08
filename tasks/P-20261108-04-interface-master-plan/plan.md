# P-20261108-04-interface-master-plan: Plan Maestro de Diseño e Interfaz

**Status:** Completed
**Created:** 2026-11-08
**Updated:** 2026-11-08
**Task list:** [todo.md](todo.md)

## Overview

Redactar `docs/design-master-plan.md` como referencia exhaustiva y normativa para la interfaz y la experiencia de uso de Nightingale. El documento nacerá en estado **Draft** y servirá para derivar planes pequeños por superficie, flujo o componente.

La aprobación autoriza solo documentación. No autoriza ejecutar diseño visual, modificar el prototipo, implementar pantallas, instalar dependencias ni cambiar React, Rust, contratos o pruebas.

## Source of Truth

- `docs/master-plan.md`: alcance, arquitectura, componentes, esqueleto UI §22 y gobierno §§26–30.
- `docs/srs.md`: usuarios, alcance y RF-01–RF-16.
- `docs/technical-architecture.md`: operación local, sincronización, estados y recuperación.
- `docs/contracts/api-v1/nightingale-api.md`: contrato vigente y semántica remota.
- `DESIGN.md`: sistema visual, Anek Latin, Aileron, Phosphor y paleta clara.
- `DESIGN-RULES.md`: tarea, contenido, consistencia y aplicación obligatoria de Laws of UX.
- `.agents/laws-of-ux/SKILL.md`: selección contextual de leyes y verificación.
- `tasks/P-20261022-01-screen-flows/plan.md`: propuesta histórica S01–S14 y UX-01–UX-12; no es aprobación visual.
- `design-prototype/`: referencia histórica exploratoria, no implementación ni fuente vigente de estilos.
- `src/`: esqueleto React actual y clientes Tauri tipados.

## Architecture Decisions

- El documento maestro será una referencia normativa, no un backlog ni una secuencia obligatoria de implementación.
- La intención funcional procede de SRS; arquitectura y contratos conservan sus propietarios; el nuevo documento gobierna UI/UX; `DESIGN.md` conserva los valores visuales.
- Toda decisión se etiquetará como requisito vigente, propuesta UX o decisión pendiente. Ninguna preferencia visual cerrará una regla de negocio abierta.
- El escritorio es la superficie normativa de este repositorio. Patient Hub se documenta como contrato de experiencia y handoff, sin crear una aplicación web aquí.
- El prototipo es evidencia histórica: conserva Inter y propuestas no validadas; no sustituye al sistema visual vigente.
- Todo plan derivado deberá declarar usuario, tarea, caso de uso Rust, datos SQLite, IPC, contexto, estados, permisos, offline/sync, accesibilidad, responsive y verificación real.

## Coverage Checklist

El maestro contendrá estas 34 secciones:

1. Función del documento.
2. Lenguaje normativo.
3. Fuentes y precedencia.
4. Alcance y exclusiones.
5. Principios de experiencia.
6. Arquitectura de información.
7. Catálogo de superficies S01–S14.
8. Recorridos extremo a extremo.
9. Acceso, preparación y shell.
10. Agenda y citas.
11. Pacientes.
12. Solicitudes y lista de espera.
13. Comunicaciones e indicadores.
14. Sincronización, configuración y ayuda.
15. Patient Hub.
16. Fundamentos visuales y tokens.
17. Tipografía, iconos y recursos.
18. Layout, densidad y adaptación.
19. Catálogo de componentes.
20. Patrones compuestos.
21. Modelo de estados.
22. Offline, sincronización y recuperación.
23. Formularios y validación.
24. Feedback y estados de superficie.
25. Permisos, privacidad y seguridad.
26. Contenido y localización.
27. Accesibilidad e interacción inclusiva.
28. Movimiento, rendimiento percibido y visualización.
29. Arquitectura frontend.
30. Verificación de interfaz.
31. Contrato para planes derivados.
32. Trazabilidad y gobierno.
33. Decisiones abiertas.
34. Resumen normativo.

La trazabilidad cubrirá RF-01–RF-16, las superficies S01–S14, los componentes y los estados relevantes.

## Dependency Order

```text
T01: fuentes y cobertura → T02: documento Draft → C01
C01 → T03: enlaces y validación → C02: revisión del usuario
```

## Task List

### Phase 1: Referencia UI/UX
- [x] T01: Registrar fuentes, cobertura y plan de redacción.
- [x] T02: Redactar el plan maestro completo.

### C01: Auditoría documental
- [x] Cobertura, coherencia, límites y estado Draft comprobados.

### Phase 2: Integración y revisión
- [x] T03: Integrar enlaces y validar documentación.

### C02: Revisión final
- [x] El usuario aprobó el maestro el 2026-11-08; no se ejecuta diseño ni implementación.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Omitir estados o excepciones | Alto | Matrices de superficie, estado, red, permiso y verificación. |
| Convertir el prototipo en diseño aprobado | Alto | Etiquetarlo como histórico y subordinarlo a DESIGN. |
| Duplicar fuentes y crear contradicciones | Alto | Enlazar propietarios y repetir solo lo necesario para UI. |
| Resolver reglas de negocio por preferencia visual | Alto | Mantener decisiones abiertas explícitas. |
| Introducir implementación implícita | Alto | Limitar archivos a documentación y no ejecutar builds. |
| Declarar aprobado el borrador | Medio | Solo el usuario podrá cambiar Draft tras revisión. |

## Open Questions

El maestro registrará, sin resolverlas, las decisiones de navegación/rutas no aprobadas, validación con usuarios, tema oscuro, colores semánticos de error/advertencia, duración de citas, datos mínimos, permisos concretos, políticas operativas, conciliación del Hub e indicadores.
