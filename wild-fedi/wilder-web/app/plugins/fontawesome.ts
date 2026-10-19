// Font Awesome, SVG-core style: we register only the icons we actually use, so
// the bundle carries a curated handful rather than a whole icon font. SSR-safe —
// `autoAddCss = false` plus importing the core CSS ourselves stops the brief
// oversize-icon flash during hydration. Exposed as both <FontAwesomeIcon> and a
// terse <Fa> alias.
import { library, config } from '@fortawesome/fontawesome-svg-core'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import '@fortawesome/fontawesome-svg-core/styles.css'
import {
  faHouse, faGlobe, faKey, faUsers, faBookOpen, faFeather,
  faBell, faCircleInfo, faMap, faBinoculars, faSchool,
  faPlus, faLeaf, faPaw, faLocationDot, faSeedling,
  faSun, faMoon, faCircleHalfStroke,
  faRightFromBracket, faRightToBracket, faBars, faXmark,
  faImage, faGear, faChartLine, faPenToSquare, faCheck,
  faUser, faTrash, faMagnifyingGlass, faFlask, faChevronRight,
  faSpinner, faSliders, faClockRotateLeft
} from '@fortawesome/free-solid-svg-icons'

config.autoAddCss = false

export default defineNuxtPlugin((nuxtApp) => {
  library.add(
    faHouse, faGlobe, faKey, faUsers, faBookOpen, faFeather,
    faBell, faCircleInfo, faMap, faBinoculars, faSchool,
    faPlus, faLeaf, faPaw, faLocationDot, faSeedling,
    faSun, faMoon, faCircleHalfStroke,
    faRightFromBracket, faRightToBracket, faBars, faXmark,
    faImage, faGear, faChartLine, faPenToSquare, faCheck,
    faUser, faTrash, faMagnifyingGlass, faFlask, faChevronRight,
    faSpinner, faSliders, faClockRotateLeft
  )
  nuxtApp.vueApp.component('FontAwesomeIcon', FontAwesomeIcon)
  nuxtApp.vueApp.component('Fa', FontAwesomeIcon)
})
