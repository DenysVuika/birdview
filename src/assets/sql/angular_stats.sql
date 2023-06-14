SELECT
    projects.name AS name,
    projects.version AS version,
    angular.version AS framework,
    COUNT(DISTINCT components.id) AS components,
    COUNT(DISTINCT modules.id) AS modules,
    COUNT(DISTINCT directives.id) AS directives,
    COUNT(DISTINCT services.id) AS services,
    COUNT(DISTINCT pipes.id) AS pipes,
    COUNT(DISTINCT dialogs.id) AS dialogs
FROM projects
    LEFT JOIN angular ON projects.id = angular.project_id
    LEFT JOIN ng_components components ON projects.id = components.project_id
    LEFT JOIN ng_modules modules ON projects.id = modules.project_id
    LEFT JOIN ng_directives directives ON projects.id = directives.project_id
    LEFT JOIN ng_services services ON projects.id = services.project_id
    LEFT JOIN ng_pipes pipes ON projects.id = pipes.project_id
    LEFT JOIN ng_dialogs dialogs ON projects.id = dialogs.project_id
GROUP BY projects.id
