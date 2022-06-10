build:
	docker build -t gmto.im/cfd_raytrace .
run:
	docker run -e CFD_CASE=zen30az000_OS7 -e AWS_BATCH_JOB_ARRAY_INDEX=0  --rm gmto.im/cfd_raytrace
push:
	aws ecr get-login-password --region us-west-2 | docker login --username AWS --password-stdin 378722409401.dkr.ecr.us-west-2.amazonaws.com
	docker tag gmto.im/cfd_raytrace:latest 378722409401.dkr.ecr.us-west-2.amazonaws.com/gmto.im/cfd_raytrace:latest
	docker push 378722409401.dkr.ecr.us-west-2.amazonaws.com/gmto.im/cfd_raytrace:latest
stack:
	aws s3 cp cfd_raytrace.yaml s3://gmto.modeling/stacks/
	aws cloudformation create-stack --stack-name CfdRayTrace --template-url https://s3-us-west-2.amazonaws.com/gmto.modeling/stacks/cfd_raytrace.yaml --region us-west-2

