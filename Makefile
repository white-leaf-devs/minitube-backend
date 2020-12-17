include vars.mk

create-domain: 
	@aws --region $(REGION) cloudsearch create-domain --domain-name $(CS_DOMAIN_NAME)

delete-domain: 
	@aws --region $(REGION) cloudsearch delete-domain --domain-name $(CS_DOMAIN_NAME)