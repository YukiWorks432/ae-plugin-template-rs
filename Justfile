PluginName       := "{{project-name}}"
BundleIdentifier := "{{bundle-identifier}}" + PluginName
BinaryName       := lowercase(PluginName)

import "./AdobePlugin.just"