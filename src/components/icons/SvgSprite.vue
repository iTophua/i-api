<template>
  <svg style="display: none">
    <symbol id="icon-get" viewBox="0 0 24 24">
      <path d="M12 4L8 8H11V16H13V8H16L12 4ZM6 10V12H10V10H6ZM6 14V16H10V14H6Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-post" viewBox="0 0 24 24">
      <path d="M8 4V16H10V12H14V16H16V4H14V10H10V4H8Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-put" viewBox="0 0 24 24">
      <path d="M8 4V16H10V12H14V16H16V4H14V10H10V4H8ZM11 6H13V8H11V6Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-delete" viewBox="0 0 24 24">
      <path d="M7 4V6H17V4H7ZM9 8V16H11V8H9ZM13 8V16H15V8H13ZM6 10V16H8V10H6ZM16 10V16H18V10H16Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-patch" viewBox="0 0 24 24">
      <path d="M12 4L8 8H11V16H13V12H15L16 13V16H18V12L14 8H11V7.41L13.29 9.71L14.71 8.29L12 5.59V4ZM11 10H13V11H11V10Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-options" viewBox="0 0 24 24">
      <path d="M12 4C14.21 4 16 5.79 16 8C16 10.21 14.21 12 12 12C9.79 12 8 10.21 8 8C8 5.79 9.79 4 12 4M12 6C10.9 6 10 6.9 10 8C10 9.1 10.9 10 12 10C13.1 10 14 9.1 14 8C14 6.9 13.1 6 12 6M8 12V16H10V14H14V16H16V12H8Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-head" viewBox="0 0 24 24">
      <path d="M8 4V16H10V10H13L15 16H17.21L14.5 8.73L17 4H14.5L13 7.5L11.5 4H8Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-success" viewBox="0 0 24 24">
      <path d="M9 12L11 14L15 10M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-error" viewBox="0 0 24 24">
      <path d="M18 6L6 18M6 6L18 18M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-warning" viewBox="0 0 24 24">
      <path d="M12 9V11M12 15H12.01M5.07183 19H18.9282C20.4678 19 21.4301 17.3333 20.6603 16L13.7321 4C12.9623 2.66667 11.0377 2.66667 10.2679 4L3.33975 16C2.56994 17.3333 3.53223 19 5.07183 19Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-info" viewBox="0 0 24 24">
      <path d="M13 16H12V12H11M12 8H12.01M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-request" viewBox="0 0 24 24">
      <path d="M4 6H20V8H4V6ZM4 11H20V13H4H11ZM4 16H20V18H4V16Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-collection" viewBox="0 0 24 24">
      <path d="M19 3H5C3.89543 3 3 3.89543 3 5V19C3 20.1046 3.89543 21 5 21H19C20.1046 21 21 20.1046 21 19V5C21 3.89543 20.1046 3 19 3ZM19 5V11H5V5H19ZM5 19V13H19V19H5Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-folder" viewBox="0 0 24 24">
      <path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H12L10 5H5C3.89543 5 3 5.89543 3 7Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-environment" viewBox="0 0 24 24">
      <path d="M12 2C13.1046 2 14 2.89543 14 4V7C14 8.10457 13.1046 9 12 9C10.8954 9 10 8.10457 10 7V4C10 2.89543 10.8954 2 12 2ZM7 10H17C18.1046 10 19 10.8954 19 12V19C19 20.1046 18.1046 21 17 21H7C5.89543 21 5 20.1046 5 19V12C5 10.8954 5.89543 10 7 10ZM12 12C10.8954 12 10 12.8954 10 14C10 15.1046 10.8954 16 12 16C13.1046 16 14 15.1046 14 14C14 12.8954 13.1046 12 12 12Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-history" viewBox="0 0 24 24">
      <path d="M12 8V12L15 15M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12ZM12 5C15.866 5 19 8.13401 19 12C19 15.866 15.866 19 12 19C8.13401 19 5 15.866 5 12C5 8.13401 8.13401 5 12 5Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-settings" viewBox="0 0 24 24">
      <path d="M12 15C13.6569 15 15 13.6569 15 12C15 10.3431 13.6569 9 12 9C10.3431 9 9 10.3431 9 12C9 13.6569 10.3431 15 12 15Z" stroke="currentColor" stroke-width="2" fill="none"/>
      <path d="M19 12C19 12.1629 18.9962 12.3251 18.9888 12.4864L21.2426 14.2426L19.8284 15.6569L17.5742 13.8907C17.0396 14.3456 16.4566 14.7369 15.8333 15.0538L16.1835 18.0538L14.1835 18.2872L13.8333 15.2872C13.2846 15.3608 12.7233 15.3981 12.1538 15.3981C11.8462 15.3981 11.5423 15.3866 11.2426 15.3638L10.8924 18.3638L8.89243 18.1304L9.24264 15.1304C8.61935 14.8135 8.03635 14.4222 7.50176 13.9673L5.24757 15.7335L3.83335 14.3193L6.08754 12.5531C6.02659 12.2328 5.97984 11.9084 5.94794 11.5804H3V9.58035H5.94794C5.97984 9.25239 6.02659 8.92804 6.08754 8.60772L3.83335 6.84154L5.24757 5.42732L7.50176 7.1935C8.03635 6.73861 8.61935 6.34732 9.24264 6.0304L8.89243 3.0304L10.8924 3.2638L11.2426 6.2638C11.5423 6.24096 11.8462 6.22949 12.1538 6.22949C12.7233 6.22949 13.2846 6.26684 13.8333 6.3404L14.1835 3.3404L16.1835 3.5738L15.8333 6.5738C16.4566 6.89068 17.0396 7.28196 17.5742 7.73685L19.8284 5.97067L21.2426 7.38489L18.9888 9.14107C18.9962 9.30239 19 9.46456 19 9.62747V12Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-send" viewBox="0 0 24 24">
      <path d="M5 12H19M5 12L11 6M5 12L11 18" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-save" viewBox="0 0 24 24">
      <path d="M8 7H16V10H16V10H8V7ZM8 13H16V17H8V13ZM19 3H5C3.89543 3 3 3.89543 3 5V19C3 20.1046 3.89543 21 5 21H19C20.1046 21 21 20.1046 21 19V5C21 3.89543 20.1046 3 19 3Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-plus" viewBox="0 0 24 24">
      <path d="M12 4V20M4 12H20" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-close" viewBox="0 0 24 24">
      <path d="M6 6L18 18M6 18L18 6" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-check" viewBox="0 0 24 24">
      <path d="M5 13L9 17L19 7" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-search" viewBox="0 0 24 24">
      <path d="M21 21L15 15M17 10C17 13.866 13.866 17 10 17C6.13401 17 3 13.866 3 10C3 6.13401 6.13401 3 10 3C13.866 3 17 6.13401 17 10Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-trash" viewBox="0 0 24 24">
      <path d="M19 7L18.1327 15.1425C18.0579 15.8446 17.4653 16.3776 16.7589 16.3776H7.24107C6.5347 16.3776 5.94208 15.8446 5.86732 15.1425L5 7M19 7V5C19 4.44772 18.5523 4 18 4H6C5.44772 4 5 4.44772 5 5V7M19 7H5M10 11V16M14 11V16" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-edit" viewBox="0 0 24 24">
      <path d="M11 4H4C3.44772 4 3 4.44772 3 5V20C3 20.5523 3.44772 21 4 21H19C19.5523 21 20 20.5523 20 20V13M18.5 2.50001C18.8978 2.10219 19.5411 2.10219 19.9389 2.50001C20.3367 2.89784 20.3367 3.54106 19.9389 3.93889L11 12.8779V16H7.76421C7.5633 16 7.36239 15.9048 7.23305 15.7497L4.25066 12.1789C4.08687 11.9823 4.11228 11.6909 4.30889 11.5271C4.34466 11.4973 4.38335 11.4712 4.42447 11.4492L11 7.93889L18.5 2.50001Z" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-copy" viewBox="0 0 24 24">
      <path d="M8 4V16C8 17.1046 8.89543 18 10 18H18C19.1046 18 20 17.1046 20 16V7.24264C20 6.71221 19.7893 6.20357 19.4142 5.82843L16.1716 2.58579C15.7964 2.21071 15.2878 2 14.7574 2H10C8.89543 2 8 2.89543 8 4ZM16 18V20C16 21.1046 15.1046 22 14 22H6C4.89543 22 4 21.1046 4 20V8C4 6.89543 4.89543 6 6 6H8" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-download" viewBox="0 0 24 24">
      <path d="M12 3V16M12 16L8 12M12 16L16 12M4 18H20" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-upload" viewBox="0 0 24 24">
      <path d="M12 16V3M12 3L8 7M12 3L16 7M4 18H20" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-refresh" viewBox="0 0 24 24">
      <path d="M4 4V9H4.58152M19.9381 11C19.4465 7.05369 16.0799 4 12 4C8.64262 4 5.76829 6.06817 4.58152 9M20 15V20H19.4185M19.4185 20C19.9101 20.9463 20 21 20 21M19.4185 20C18.9269 20.9463 18 21 18 21M19.4185 20C15.2899 20 15.0535 20.9463 15.0535 20.9463M4.06189 13C4.55348 12.0537 5 12 5 12M4.06189 13C4.55348 13.9463 5.44652 15 6 15M4.06189 13C3.5703 13.9463 4 14 4 14M5 12C5.55348 10.9463 6.62172 10 8 10" stroke="currentColor" stroke-width="2" fill="none"/>
    </symbol>
    
    <symbol id="icon-play" viewBox="0 0 24 24">
      <path d="M8 5V19L19 12L8 5Z" fill="currentColor"/>
    </symbol>
    
    <symbol id="icon-stop" viewBox="0 0 24 24">
      <rect x="6" y="6" width="12" height="12" fill="currentColor"/>
    </symbol>
  </svg>
</template>
