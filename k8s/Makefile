PROCESS=osmosis
KUBECONTEXT=$(shell kubectl config current-context)

SHELL := /bin/bash

# Set cluster based on kubeconfig context
# TODO: need to make switching between different namespaces as seemless as possible
CLUSTER=$(KUBECONTEXT)

###############################################################################
###                              Local Setup                                ###
###############################################################################

install-kind:
ifeq ($(shell uname -s), Linux)
	curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.15.0/kind-linux-amd64
	chmod +x ./kind
	sudo mv ./kind /usr/local/bin/kind
else
	brew install kind
endif

kind_check := $(shell command -v kind 2> /dev/null)
setup-kind:
ifndef kind_check
	echo "No kind in $(PATH), installing kind..."
	$(MAKE) install-kind
endif
	echo "kind already insteall.. setting up kind cluster"
	kind create cluster

install-kubectl:
ifeq ($(shell uname -s), Linux)
	curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
	chmod +x ./kubectl
	sudo mv ./kubectl /usr/local/bin/kubectl
else
	brew install kubectl
endif

kubectl_check := $(shell command -v kubectl 2> /dev/null)
setup-kubectl:
ifndef kubectl_check
	$(MAKE) install-kubectl
endif
	echo "kubectl already installed"

setup: setup-kubectl setup-kind

clean-kind:
	kind delete cluster

clean: clean-kind
	-@rm ./kind ./kubectl 2> /dev/null

###############################################################################
###                              K8s commands                               ###
###############################################################################

apply:
	kubectl apply -k $(CLUSTER)/$(PROCESS)

apply-all:
	$(MAKE) apply PROCESS=osmosis
	$(MAKE) apply PROCESS=juno
	$(MAKE) apply PROCESS=relayer

delete:
	kubectl delete -k $(CLUSTER)/$(PROCESS) --force

delete-all:
	-@$(MAKE) delete PROCESS=osmosis
	-@$(MAKE) delete PROCESS=juno
	-@$(MAKE) delete PROCESS=relayer
