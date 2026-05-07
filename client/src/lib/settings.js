import { DEFAULT_ALIAS_TEMPLATE } from './utils.js';
export { DEFAULT_ALIAS_TEMPLATE };

const ALIAS_TEMPLATE_KEY = 'flaremask_alias_template';

export function getAliasTemplate() {
    return localStorage.getItem(ALIAS_TEMPLATE_KEY) ?? DEFAULT_ALIAS_TEMPLATE;
}

export function saveAliasTemplate(template) {
    localStorage.setItem(ALIAS_TEMPLATE_KEY, template);
}
