pipeline {
    agent any

    options {
        timestamps()
        disableConcurrentBuilds()
    }

    environment {
        REGISTRY = "qd-registries-registry-vpc.cn-qingdao.cr.aliyuncs.com/kaiheila"
        HELM_CHART_DIR = "/opt/helm_charts"
        PROD_KUBE_CONFIG = "/opt/helm_charts/.kube/qd_k8s_00"
        IMAGE_TAG = "${env.BUILD_NUMBER}"
        BACKEND_IMAGE = "${REGISTRY}/kook-ai/backend"
        FRONTEND_IMAGE = "${REGISTRY}/kook-ai/frontend"
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Build Backend') {
            steps {
                sh "docker build -t ${BACKEND_IMAGE}:${IMAGE_TAG} -f backend/Dockerfile backend"
            }
        }

        stage('Build Frontend') {
            steps {
                sh "docker build -t ${FRONTEND_IMAGE}:${IMAGE_TAG} -f frontend/Dockerfile frontend"
            }
        }

        stage('Push Images') {
            steps {
                sh "docker push ${BACKEND_IMAGE}:${IMAGE_TAG}"
                sh "docker push ${FRONTEND_IMAGE}:${IMAGE_TAG}"
            }
        }

        stage('Helm Deploy') {
            steps {
                sh """
                helm upgrade --install kook-ai ${HELM_CHART_DIR}/kook-ai \
                  --namespace kook-ai \
                  --create-namespace \
                  --kubeconfig ${PROD_KUBE_CONFIG} \
                  -f ${HELM_CHART_DIR}/kook-ai/values-production.yaml \
                  --set backend.image.tag=${IMAGE_TAG} \
                  --set frontend.image.tag=${IMAGE_TAG}
                """
            }
        }
    }
}
