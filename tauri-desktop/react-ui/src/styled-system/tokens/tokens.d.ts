/* eslint-disable */
export type Token = `aspectRatios.${AspectRatioToken}` | `borders.${BorderToken}` | `easings.${EasingToken}` | `durations.${DurationToken}` | `radii.${RadiusToken}` | `shadows.${ShadowToken}` | `colors.${ColorToken}` | `blurs.${BlurToken}` | `sizes.${SizeToken}` | `animations.${AnimationToken}` | `fontSizes.${FontSizeToken}` | `fontWeights.${FontWeightToken}` | `lineHeights.${LineHeightToken}` | `letterSpacings.${LetterSpacingToken}` | `fonts.${FontToken}` | `zIndex.${ZIndexToken}` | `spacing.${SpacingToken}` | `breakpoints.${BreakpointToken}`

export type ColorPalette = "current" | "black" | "white" | "transparent" | "rose" | "pink" | "fuchsia" | "purple" | "violet" | "indigo" | "blue" | "sky" | "cyan" | "teal" | "emerald" | "green" | "lime" | "yellow" | "amber" | "orange" | "red" | "neutral" | "stone" | "zinc" | "gray" | "slate" | "brand" | "status" | "status.success" | "status.warning" | "status.error" | "status.attention" | "text" | "background" | "border" | "shadow" | "icon" | "eventKind" | "taskKind" | "companyStatus" | "companyStatus.inProcess" | "companyStatus.recruiting" | "companyStatus.offer" | "companyStatus.rejected" | "companyStatus.closed" | "deadlineUrgency" | "calendar"

export type AspectRatioToken = "square" | "landscape" | "portrait" | "wide" | "ultrawide" | "golden"

export type BorderToken = "none"

export type EasingToken = "default" | "linear" | "in" | "out" | "in-out"

export type DurationToken = "fastest" | "faster" | "fast" | "normal" | "slow" | "slower" | "slowest"

export type RadiusToken = "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "full"

export type ShadowToken = "2xs" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "inset-2xs" | "inset-xs" | "inset-sm"

export type ColorToken = "current" | "black" | "white" | "transparent" | "rose.50" | "rose.100" | "rose.200" | "rose.300" | "rose.400" | "rose.500" | "rose.600" | "rose.700" | "rose.800" | "rose.900" | "rose.950" | "pink.50" | "pink.100" | "pink.200" | "pink.300" | "pink.400" | "pink.500" | "pink.600" | "pink.700" | "pink.800" | "pink.900" | "pink.950" | "fuchsia.50" | "fuchsia.100" | "fuchsia.200" | "fuchsia.300" | "fuchsia.400" | "fuchsia.500" | "fuchsia.600" | "fuchsia.700" | "fuchsia.800" | "fuchsia.900" | "fuchsia.950" | "purple.50" | "purple.100" | "purple.200" | "purple.300" | "purple.400" | "purple.500" | "purple.600" | "purple.700" | "purple.800" | "purple.900" | "purple.950" | "violet.50" | "violet.100" | "violet.200" | "violet.300" | "violet.400" | "violet.500" | "violet.600" | "violet.700" | "violet.800" | "violet.900" | "violet.950" | "indigo.50" | "indigo.100" | "indigo.200" | "indigo.300" | "indigo.400" | "indigo.500" | "indigo.600" | "indigo.700" | "indigo.800" | "indigo.900" | "indigo.950" | "blue.50" | "blue.100" | "blue.200" | "blue.300" | "blue.400" | "blue.500" | "blue.600" | "blue.700" | "blue.800" | "blue.900" | "blue.950" | "sky.50" | "sky.100" | "sky.200" | "sky.300" | "sky.400" | "sky.500" | "sky.600" | "sky.700" | "sky.800" | "sky.900" | "sky.950" | "cyan.50" | "cyan.100" | "cyan.200" | "cyan.300" | "cyan.400" | "cyan.500" | "cyan.600" | "cyan.700" | "cyan.800" | "cyan.900" | "cyan.950" | "teal.50" | "teal.100" | "teal.200" | "teal.300" | "teal.400" | "teal.500" | "teal.600" | "teal.700" | "teal.800" | "teal.900" | "teal.950" | "emerald.50" | "emerald.100" | "emerald.200" | "emerald.300" | "emerald.400" | "emerald.500" | "emerald.600" | "emerald.700" | "emerald.800" | "emerald.900" | "emerald.950" | "green.50" | "green.100" | "green.200" | "green.300" | "green.400" | "green.500" | "green.600" | "green.700" | "green.800" | "green.900" | "green.950" | "lime.50" | "lime.100" | "lime.200" | "lime.300" | "lime.400" | "lime.500" | "lime.600" | "lime.700" | "lime.800" | "lime.900" | "lime.950" | "yellow.50" | "yellow.100" | "yellow.200" | "yellow.300" | "yellow.400" | "yellow.500" | "yellow.600" | "yellow.700" | "yellow.800" | "yellow.900" | "yellow.950" | "amber.50" | "amber.100" | "amber.200" | "amber.300" | "amber.400" | "amber.500" | "amber.600" | "amber.700" | "amber.800" | "amber.900" | "amber.950" | "orange.50" | "orange.100" | "orange.200" | "orange.300" | "orange.400" | "orange.500" | "orange.600" | "orange.700" | "orange.800" | "orange.900" | "orange.950" | "red.50" | "red.100" | "red.200" | "red.300" | "red.400" | "red.500" | "red.600" | "red.700" | "red.800" | "red.900" | "red.950" | "neutral.50" | "neutral.100" | "neutral.200" | "neutral.300" | "neutral.400" | "neutral.500" | "neutral.600" | "neutral.700" | "neutral.800" | "neutral.900" | "neutral.950" | "stone.50" | "stone.100" | "stone.200" | "stone.300" | "stone.400" | "stone.500" | "stone.600" | "stone.700" | "stone.800" | "stone.900" | "stone.950" | "zinc.50" | "zinc.100" | "zinc.200" | "zinc.300" | "zinc.400" | "zinc.500" | "zinc.600" | "zinc.700" | "zinc.800" | "zinc.900" | "zinc.950" | "gray.50" | "gray.100" | "gray.200" | "gray.300" | "gray.400" | "gray.500" | "gray.600" | "gray.700" | "gray.800" | "gray.900" | "gray.950" | "slate.50" | "slate.100" | "slate.200" | "slate.300" | "slate.400" | "slate.500" | "slate.600" | "slate.700" | "slate.800" | "slate.900" | "slate.950" | "brand.main" | "brand.light" | "brand.hover" | "brand.soft" | "brand.softBorder" | "status.success.main" | "status.success.soft" | "status.success.softBorder" | "status.warning.main" | "status.warning.soft" | "status.warning.softBorder" | "status.error.main" | "status.error.soft" | "status.error.softBorder" | "status.attention.main" | "status.attention.soft" | "status.attention.softBorder" | "text.main" | "text.sub" | "text.placeholder" | "text.disabled" | "background.main" | "background.sub" | "background.hover" | "background.overlay" | "border.primary" | "border.secondary" | "border.focus" | "shadow.primary" | "shadow.secondary" | "icon.primary" | "icon.secondary" | "icon.disabled" | "eventKind.meeting" | "eventKind.infoSession" | "eventKind.interview" | "taskKind.entry" | "taskKind.esDraft" | "taskKind.aptitude" | "taskKind.webTest" | "taskKind.codingTest" | "companyStatus.inProcess.main" | "companyStatus.inProcess.soft" | "companyStatus.inProcess.softBorder" | "companyStatus.recruiting.main" | "companyStatus.recruiting.soft" | "companyStatus.recruiting.softBorder" | "companyStatus.offer.main" | "companyStatus.offer.soft" | "companyStatus.offer.softBorder" | "companyStatus.rejected.main" | "companyStatus.rejected.soft" | "companyStatus.rejected.softBorder" | "companyStatus.closed.main" | "companyStatus.closed.soft" | "companyStatus.closed.softBorder" | "deadlineUrgency.tomorrow" | "deadlineUrgency.inTwoDays" | "deadlineUrgency.inThreeDays" | "calendar.saturday" | "calendar.sunday" | "colorPalette" | "colorPalette.50" | "colorPalette.100" | "colorPalette.200" | "colorPalette.300" | "colorPalette.400" | "colorPalette.500" | "colorPalette.600" | "colorPalette.700" | "colorPalette.800" | "colorPalette.900" | "colorPalette.950" | "colorPalette.main" | "colorPalette.light" | "colorPalette.hover" | "colorPalette.soft" | "colorPalette.softBorder" | "colorPalette.success.main" | "colorPalette.success.soft" | "colorPalette.success.softBorder" | "colorPalette.warning.main" | "colorPalette.warning.soft" | "colorPalette.warning.softBorder" | "colorPalette.error.main" | "colorPalette.error.soft" | "colorPalette.error.softBorder" | "colorPalette.attention.main" | "colorPalette.attention.soft" | "colorPalette.attention.softBorder" | "colorPalette.sub" | "colorPalette.placeholder" | "colorPalette.disabled" | "colorPalette.overlay" | "colorPalette.primary" | "colorPalette.secondary" | "colorPalette.focus" | "colorPalette.meeting" | "colorPalette.infoSession" | "colorPalette.interview" | "colorPalette.entry" | "colorPalette.esDraft" | "colorPalette.aptitude" | "colorPalette.webTest" | "colorPalette.codingTest" | "colorPalette.inProcess.main" | "colorPalette.inProcess.soft" | "colorPalette.inProcess.softBorder" | "colorPalette.recruiting.main" | "colorPalette.recruiting.soft" | "colorPalette.recruiting.softBorder" | "colorPalette.offer.main" | "colorPalette.offer.soft" | "colorPalette.offer.softBorder" | "colorPalette.rejected.main" | "colorPalette.rejected.soft" | "colorPalette.rejected.softBorder" | "colorPalette.closed.main" | "colorPalette.closed.soft" | "colorPalette.closed.softBorder" | "colorPalette.tomorrow" | "colorPalette.inTwoDays" | "colorPalette.inThreeDays" | "colorPalette.saturday" | "colorPalette.sunday"

export type BlurToken = "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl"

export type SizeToken = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12" | "14" | "16" | "20" | "24" | "28" | "32" | "36" | "40" | "44" | "48" | "52" | "56" | "60" | "64" | "72" | "80" | "96" | "0.5" | "1.5" | "2.5" | "3.5" | "4.5" | "5.5" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl" | "6xl" | "7xl" | "8xl" | "prose" | "full" | "min" | "max" | "fit" | "breakpoint-2xl" | "breakpoint-sm" | "breakpoint-md" | "breakpoint-lg" | "breakpoint-xl"

export type AnimationToken = "spin" | "ping" | "pulse" | "bounce"

export type FontSizeToken = "2xs" | "md" | "xs" | "sm" | "base" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl" | "6xl" | "7xl" | "8xl" | "9xl"

export type FontWeightToken = "light" | "extrabold" | "black" | "thin" | "extralight" | "regular" | "normal" | "medium" | "semibold" | "bold"

export type LineHeightToken = "none" | "tight" | "snug" | "normal" | "relaxed" | "loose"

export type LetterSpacingToken = "tighter" | "tight" | "normal" | "wide" | "wider" | "widest"

export type FontToken = "sans" | "serif" | "mono"

export type ZIndexToken = "modal" | "floating" | "header" | "drawer" | "footer" | "front" | "default" | "background"

export type SpacingToken = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12" | "14" | "16" | "20" | "24" | "28" | "32" | "36" | "40" | "44" | "48" | "52" | "56" | "60" | "64" | "72" | "80" | "96" | "0.5" | "1.5" | "2.5" | "3.5" | "4.5" | "5.5" | "xs" | "sm" | "md" | "lg" | "xl" | "-1" | "-2" | "-3" | "-4" | "-5" | "-6" | "-7" | "-8" | "-9" | "-10" | "-11" | "-12" | "-14" | "-16" | "-20" | "-24" | "-28" | "-32" | "-36" | "-40" | "-44" | "-48" | "-52" | "-56" | "-60" | "-64" | "-72" | "-80" | "-96" | "-0.5" | "-1.5" | "-2.5" | "-3.5" | "-4.5" | "-5.5" | "-xs" | "-sm" | "-md" | "-lg" | "-xl"

export type BreakpointToken = "2xl" | "sm" | "md" | "lg" | "xl"

export type Tokens = {
		aspectRatios: AspectRatioToken
		borders: BorderToken
		easings: EasingToken
		durations: DurationToken
		radii: RadiusToken
		shadows: ShadowToken
		colors: ColorToken
		blurs: BlurToken
		sizes: SizeToken
		animations: AnimationToken
		fontSizes: FontSizeToken
		fontWeights: FontWeightToken
		lineHeights: LineHeightToken
		letterSpacings: LetterSpacingToken
		fonts: FontToken
		zIndex: ZIndexToken
		spacing: SpacingToken
		breakpoints: BreakpointToken
} & { [token: string]: never }

export type TokenCategory = "aspectRatios" | "zIndex" | "opacity" | "colors" | "fonts" | "fontSizes" | "fontWeights" | "lineHeights" | "letterSpacings" | "sizes" | "cursor" | "shadows" | "spacing" | "radii" | "borders" | "borderWidths" | "durations" | "easings" | "animations" | "blurs" | "gradients" | "breakpoints" | "assets"