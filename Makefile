zip:
	rm ../CFD.zip
	zip -r ../CFD.zip *
	ls -lh ../CFD.zip

upload:
	aws s3 cp ../CFD.zip s3://gmto.rconan/
	aws lambda update-function-code --function-name CFD2 --s3-bucket gmto.rconan --s3-key CFD.zip

all: zip upload
