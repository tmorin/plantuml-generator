pub const TEMPLATE: &str = r##"@startuml
{%- block header %}{% endblock header %}

' by default the inclusion mode is remote
!if (%not(%variable_exists("$INCLUSION_MODE")))
    !global $INCLUSION_MODE="remote"
!endif

!if ($INCLUSION_MODE == "remote")
    !if (%not(%variable_exists("$LIB_BASE_LOCATION")))
        !global $LIB_BASE_LOCATION="{{ data.remote_url }}"
    !endif
!else
    !if (%not(%variable_exists("$LIB_BASE_LOCATION")))
        !global $LIB_BASE_LOCATION="."
    !endif
!endif

!if (%not(%variable_exists("$IMAGE_BASE_PATH"))) && (%variable_exists("$LIB_BASE_LOCATION"))
    !global $IMAGE_BASE_PATH=$LIB_BASE_LOCATION + "/"
!endif

' constants
{% block constants %}
!global $ICON_FORMAT="{{ data.icon_format }}"
!global $TEXT_WIDTH_MAX={{ data.text_width_max }}
!global $MSG_WIDTH_MAX={{ data.msg_width_max }}
!global $FONT_SIZE_XS={{ data.font_size_xs }}
!global $FONT_SIZE_SM={{ data.font_size_sm }}
!global $FONT_SIZE_MD={{ data.font_size_md }}
!global $FONT_SIZE_LG={{ data.font_size_lg }}
!global $FONT_COLOR="{{ data.font_color }}"
!global $FONT_COLOR_LIGHT="{{ data.font_color_light }}"
{% endblock contants %}

' Styles
{% block styles %}
hide stereotype
skinparam wrapWidth $TEXT_WIDTH_MAX
skinparam maxMessageSize $MSG_WIDTH_MAX
skinparam DefaultFontSize $FONT_SIZE_SM
skinparam DefaultFontColor $FONT_COLOR
{% endblock styles %}

' Title
{% block procedure_styles %}
!procedure Title($title="", $subtitle="", $version="", $date="")
    !$s="$title"
    !$s_d="Last modified: " + %date("yyyy-MM-dd'T'HH:mm:ss")
    !$s_v=""
    !if ($date != "")
        !$s_d="Last modified: " + $date
    !endif
    !if ($version != "")
        !$s_v=" | version: " + $version
    !endif
    !$s=$s_d + $s_v
    left header
    !if ($title != "")
        <color:$FONT_COLOR><size:$FONT_SIZE_LG>$title</size></color>
    !endif
    !if ($subtitle)
        <color:$FONT_COLOR><size:$FONT_SIZE_MD>$subtitle</size></color>
    !endif
        <color:$FONT_COLOR_LIGHT><size:$FONT_SIZE_XS>$s</size></color>
    end header
!endprocedure
{% endblock procedure_styles %}

' getIcon()
{% block function_getIcon %}
!function getIcon($name)
    !return $IMAGE_BASE_PATH/$name.$ICON_FORMAT
!endfunction
{% endblock function_getIcon %}

' include()
{% block procedure_include %}
!procedure include($resource)
    !include $LIB_BASE_LOCATION/$resource.puml
!endprocedure
{% endblock procedure_include %}

' Relationship
{% block procedure_relationship %}
!procedure Relationship($label="", $tech="")
    !if ($label != '' && $tech != '')
        $label\n<size:$FONT_SIZE_XS><color:$FONT_COLOR_LIGHT>[$tech]</color></size>
    !elseif ($label != '')
        $label
    !else
        <size:$FONT_SIZE_XS><color:$FONT_COLOR_LIGHT>[$tech]</color></size>
    !endif
!endprocedure
{% endblock procedure_relationship %}

' IconElement
{% block procedure_IconElement %}
!procedure IconElement($id, $stereotype, $icon, $name="", $tech="")
    !local $H="<img:" + getIcon($icon)+ ">"
    !if ($name != "")
        !$H=$H + "\n" + $name
    !endif
    !if ($tech != "")
        !$H=$H + "\n" + "<size:" + $FONT_SIZE_XS + "><color:" + $FONT_COLOR_LIGHT + ">[" + $tech + "]</color></size>"
    !endif
    card $id <<$stereotype>> as "$H"
!endprocedure
{% endblock procedure_IconElement %}

' IconCardElement
{% block procedure_IconCardElement %}
!procedure IconCardElement($id, $stereotype, $sprite, $techName="", $funcName="", $content="")
    !local $V=""
    !local $H=""
    !local $S="<color:" + $FONT_COLOR + ">" + $sprite + " </color>"
    !local $F=""
    !if ($techName != "") && ($funcName != "")
        !$ST="<size:" + $FONT_SIZE_MD + ">" + "<color:" + $FONT_COLOR_LIGHT + ">" + $techName + "</color>" + "</size>"
        !$T="<size:" + $FONT_SIZE_MD + ">" + "<color:" + $FONT_COLOR + ">" + $funcName + "</color>" + "</size>"
        !$H=$T + "\l" + $S + $ST
        !$V=$V + $H
    !elseif ($techName != "")
        !$ST="<size:" + $FONT_SIZE_MD + ">" + "<color:" + $FONT_COLOR_LIGHT + ">" + $techName + "</color>" + "</size>"
        !$H=$S + $ST
        !$V=$V + $H
    !elseif ($funcName != "")
        !$T="<size:" + $FONT_SIZE_MD + ">" + "<color:" + $FONT_COLOR + ">" + $funcName + "</color>" + "</size>"
        !$H=$S + $T
        !$V=$V + $H
    !endif
    !if ($H != "") && ($content != "")
        !$F="\n----\n" + $content
        !$V=$V + $F
    !elseif ($content != "")
        !$F=$content
        !$V=$S + "\n" + $F
    !endif
    Rectangle $id <<$stereotype>> as "$V"
!endprocedure
{% endblock procedure_IconCardElement %}

' IconGroupElement
{% block procedure_IconGroupElement %}
!procedure IconGroupElement($id, $stereotype, $sprite, $name="", $tech="")
    !local $V=$sprite + " "
    !if ($name != "") && ($tech != "")
        !$V=$V + $name + "\n" + "<size:" + $FONT_SIZE_XS + "><color:" + $FONT_COLOR_LIGHT + ">[" + $tech + "]</color></size>"
    !elseif ($name != "")
        !$V=$V + $name
    !elseif ($tech != "")
        !$V=$V + "<size:" + $FONT_SIZE_XS + "><color:" + $FONT_COLOR_LIGHT + ">[" + $tech + "]</color></size>"
    !endif
    Rectangle $id <<$stereotype>> as "$V"
!endprocedure
{% endblock procedure_IconGroupElement %}

' GroupElement
{% block procedure_GroupElement %}
!procedure GroupElement($id, $stereotype, $name="", $tech="")
    !local $V=""
    !if ($name != "") && ($tech != "")
        !$V=$V + $name + "\n" + "<size:" + $FONT_SIZE_XS + "><color:" + $FONT_COLOR_LIGHT + ">[" + $tech + "]</color></size>"
    !elseif ($name != "")
        !$V=$V + $name
    !elseif ($tech != "")
        !$V=$V + "<size:" + $FONT_SIZE_XS + "><color:" + $FONT_COLOR_LIGHT + ">[" + $tech + "]</color></size>"
    !endif
    !if ($V != "")
        Rectangle $id <<$stereotype>> as "$V"
    !else
        Rectangle $id <<$stereotype>>
    !endif
!endprocedure
{% endblock procedure_GroupElement -%}

{%- block footer %}{% endblock footer %}
@enduml"##;
